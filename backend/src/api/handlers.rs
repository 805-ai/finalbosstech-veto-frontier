// API request handlers
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, error};
use uuid::Uuid;

use super::{ApiError, AppState};
use crate::{
    crypto::{ReceiptData, SignedReceipt},
    db::{
        models::{PointerStatus, ReceiptOperation},
        queries::*,
    },
    enforcement::enforce_pointer_access,
};

// ============================================================================
// HEALTH CHECK
// ============================================================================

pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "veto-frontier-backend",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

// ============================================================================
// CREATE POINTER
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreatePointerRequest {
    pub subject_id: String,
    pub content_hash: String,
    #[serde(default)]
    pub encrypted_payload: Option<String>, // Base64 encoded
}

#[derive(Debug, Serialize)]
pub struct CreatePointerResponse {
    pub pointer_id: Uuid,
    pub data_id: Uuid,
    pub status: String,
    pub receipt: ReceiptInfo,
}

#[derive(Debug, Serialize)]
pub struct ReceiptInfo {
    pub receipt_hash: String,
    pub signature: String, // Base64 encoded
    pub signature_algorithm: String,
    pub timestamp: String,
}

pub async fn create_pointer(
    State(state): State<AppState>,
    Json(req): Json<CreatePointerRequest>,
) -> Result<(StatusCode, Json<CreatePointerResponse>), ApiError> {
    info!("Creating pointer for subject: {}", req.subject_id);

    let org_id = state.config.default_org_id;

    // Decode payload if provided
    let payload_bytes = if let Some(ref payload_base64) = req.encrypted_payload {
        Some(
            data_encoding::BASE64
                .decode(payload_base64.as_bytes())
                .map_err(|e| ApiError::BadRequest(format!("Invalid base64 payload: {}", e)))?,
        )
    } else {
        None
    };

    // 1. Store data
    let data = create_data_store(
        &state.db_pool,
        org_id,
        &req.subject_id,
        &req.content_hash,
        payload_bytes.as_deref(),
    )
    .await?;

    info!("Created data_store entry: {}", data.data_id);

    // 2. Create pointer
    let pointer = create_pointer(
        &state.db_pool,
        org_id,
        data.data_id,
        &req.subject_id,
    )
    .await?;

    info!("Created pointer: {}", pointer.pointer_id);

    // 3. Generate signed receipt
    let receipt_data = ReceiptData::new(
        pointer.pointer_id,
        ReceiptOperation::Create,
        req.subject_id.clone(),
        None, // First receipt, no previous hash
        json!({"content_hash": req.content_hash}),
    );

    let signed_receipt = receipt_data.sign(&state.keypair)?;

    // 4. Store receipt in database
    create_governance_receipt(
        &state.db_pool,
        pointer.pointer_id,
        org_id,
        ReceiptOperation::Create,
        signed_receipt.receipt_json.clone(),
        &signed_receipt.receipt_hash,
        &signed_receipt.signature,
        &signed_receipt.signature_algorithm,
        None,
    )
    .await?;

    info!("Created governance receipt for pointer: {}", pointer.pointer_id);

    // 5. Create audit log
    create_audit_log(
        &state.db_pool,
        Some(org_id),
        Some(pointer.pointer_id),
        None,
        "pointer_created",
        json!({
            "subject_id": req.subject_id,
            "content_hash": req.content_hash,
        }),
        None,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatePointerResponse {
            pointer_id: pointer.pointer_id,
            data_id: data.data_id,
            status: "active".to_string(),
            receipt: ReceiptInfo {
                receipt_hash: signed_receipt.receipt_hash,
                signature: data_encoding::BASE64.encode(&signed_receipt.signature),
                signature_algorithm: signed_receipt.signature_algorithm,
                timestamp: pointer.created_at.to_rfc3339(),
            },
        }),
    ))
}

// ============================================================================
// RESOLVE POINTER
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ResolvePointerResponse {
    pub pointer_id: Uuid,
    pub data_id: Uuid,
    pub subject_id: String,
    pub content_hash: String,
    pub status: String,
    pub created_at: String,
    pub receipt: ReceiptInfo,
}

pub async fn resolve_pointer(
    State(state): State<AppState>,
    Path(pointer_id): Path<Uuid>,
) -> Result<Json<ResolvePointerResponse>, ApiError> {
    info!("Resolving pointer: {}", pointer_id);

    // 1. Get pointer
    let pointer = get_pointer(&state.db_pool, pointer_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pointer not found".to_string()))?;

    // 2. ENFORCE: Check if pointer is orphaned
    enforce_pointer_access(&pointer)?;

    // 3. Get associated data
    let data = get_data_store(&state.db_pool, pointer.data_id)
        .await?
        .ok_or_else(|| ApiError::Internal("Data not found for pointer".to_string()))?;

    // 4. Generate resolve receipt
    let prev_hash = get_latest_receipt_hash(&state.db_pool, pointer_id).await?;

    let receipt_data = ReceiptData::new(
        pointer.pointer_id,
        ReceiptOperation::Resolve,
        pointer.subject_id.clone(),
        prev_hash.clone(),
        json!({"data_id": data.data_id}),
    );

    let signed_receipt = receipt_data.sign(&state.keypair)?;

    // 5. Store resolve receipt
    create_governance_receipt(
        &state.db_pool,
        pointer.pointer_id,
        pointer.org_id,
        ReceiptOperation::Resolve,
        signed_receipt.receipt_json.clone(),
        &signed_receipt.receipt_hash,
        &signed_receipt.signature,
        &signed_receipt.signature_algorithm,
        prev_hash.as_deref(),
    )
    .await?;

    info!("Resolved pointer successfully: {}", pointer_id);

    Ok(Json(ResolvePointerResponse {
        pointer_id: pointer.pointer_id,
        data_id: data.data_id,
        subject_id: pointer.subject_id,
        content_hash: data.content_hash,
        status: match pointer.status {
            PointerStatus::Active => "active".to_string(),
            PointerStatus::Orphaned => "orphaned".to_string(),
        },
        created_at: pointer.created_at.to_rfc3339(),
        receipt: ReceiptInfo {
            receipt_hash: signed_receipt.receipt_hash,
            signature: data_encoding::BASE64.encode(&signed_receipt.signature),
            signature_algorithm: signed_receipt.signature_algorithm,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    }))
}

// ============================================================================
// ORPHAN POINTER (VETO)
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct OrphanPointerRequest {
    pub pointer_id: Uuid,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrphanPointerResponse {
    pub pointer_id: Uuid,
    pub status: String,
    pub orphaned_at: String,
    pub receipt: ReceiptInfo,
}

pub async fn orphan_pointer(
    State(state): State<AppState>,
    Json(req): Json<OrphanPointerRequest>,
) -> Result<Json<OrphanPointerResponse>, ApiError> {
    info!("Orphaning pointer: {}", req.pointer_id);

    // 1. Get current pointer
    let pointer_before = get_pointer(&state.db_pool, req.pointer_id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Pointer not found".to_string()))?;

    // 2. Check if already orphaned
    if matches!(pointer_before.status, PointerStatus::Orphaned) {
        return Err(ApiError::BadRequest(
            "Pointer is already orphaned".to_string(),
        ));
    }

    // 3. Orphan the pointer
    let orphaned_pointer = orphan_pointer(
        &state.db_pool,
        req.pointer_id,
        req.reason.as_deref(),
    )
    .await?;

    info!("Pointer orphaned: {}", req.pointer_id);

    // 4. Generate orphan receipt with chain link
    let prev_hash = get_latest_receipt_hash(&state.db_pool, req.pointer_id).await?;

    let receipt_data = ReceiptData::new(
        orphaned_pointer.pointer_id,
        ReceiptOperation::Orphan,
        orphaned_pointer.subject_id.clone(),
        prev_hash.clone(),
        json!({
            "reason": req.reason.clone().unwrap_or_else(|| "user_consent_revoked".to_string()),
            "orphaned_at": orphaned_pointer.orphaned_at,
        }),
    );

    let signed_receipt = receipt_data.sign(&state.keypair)?;

    // 5. Store orphan receipt
    create_governance_receipt(
        &state.db_pool,
        orphaned_pointer.pointer_id,
        orphaned_pointer.org_id,
        ReceiptOperation::Orphan,
        signed_receipt.receipt_json.clone(),
        &signed_receipt.receipt_hash,
        &signed_receipt.signature,
        &signed_receipt.signature_algorithm,
        prev_hash.as_deref(),
    )
    .await?;

    // 6. Audit log
    create_audit_log(
        &state.db_pool,
        Some(orphaned_pointer.org_id),
        Some(orphaned_pointer.pointer_id),
        None,
        "pointer_orphaned",
        json!({
            "subject_id": orphaned_pointer.subject_id,
            "reason": req.reason,
        }),
        None,
    )
    .await?;

    info!("Orphan receipt created for pointer: {}", req.pointer_id);

    Ok(Json(OrphanPointerResponse {
        pointer_id: orphaned_pointer.pointer_id,
        status: "orphaned".to_string(),
        orphaned_at: orphaned_pointer
            .orphaned_at
            .unwrap()
            .to_rfc3339(),
        receipt: ReceiptInfo {
            receipt_hash: signed_receipt.receipt_hash,
            signature: data_encoding::BASE64.encode(&signed_receipt.signature),
            signature_algorithm: signed_receipt.signature_algorithm,
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    }))
}

// ============================================================================
// GET RECEIPTS
// ============================================================================

#[derive(Debug, Serialize)]
pub struct GetReceiptsResponse {
    pub pointer_id: Uuid,
    pub receipts: Vec<ReceiptSummary>,
}

#[derive(Debug, Serialize)]
pub struct ReceiptSummary {
    pub operation: String,
    pub receipt_hash: String,
    pub signature: String,
    pub prev_hash: Option<String>,
    pub timestamp: String,
}

pub async fn get_receipts(
    State(state): State<AppState>,
    Path(pointer_id): Path<Uuid>,
) -> Result<Json<GetReceiptsResponse>, ApiError> {
    info!("Getting receipts for pointer: {}", pointer_id);

    let receipts = get_receipts_by_pointer(&state.db_pool, pointer_id).await?;

    let receipt_summaries: Vec<ReceiptSummary> = receipts
        .into_iter()
        .map(|r| ReceiptSummary {
            operation: match r.operation {
                ReceiptOperation::Create => "create".to_string(),
                ReceiptOperation::Resolve => "resolve".to_string(),
                ReceiptOperation::Orphan => "orphan".to_string(),
            },
            receipt_hash: r.receipt_hash,
            signature: data_encoding::BASE64.encode(&r.signature),
            prev_hash: r.prev_hash,
            timestamp: r.timestamp.to_rfc3339(),
        })
        .collect();

    Ok(Json(GetReceiptsResponse {
        pointer_id,
        receipts: receipt_summaries,
    }))
}

// ============================================================================
// GET AUDIT TRAIL
// ============================================================================

#[derive(Debug, Serialize)]
pub struct GetAuditTrailResponse {
    pub subject_id: String,
    pub total_pointers: usize,
    pub active_pointers: usize,
    pub orphaned_pointers: usize,
    pub audit_events: Vec<AuditEventSummary>,
}

#[derive(Debug, Serialize)]
pub struct AuditEventSummary {
    pub event_type: String,
    pub timestamp: String,
    pub pointer_id: Option<Uuid>,
    pub event_data: serde_json::Value,
}

pub async fn get_audit_trail(
    State(state): State<AppState>,
    Path(subject_id): Path<String>,
) -> Result<Json<GetAuditTrailResponse>, ApiError> {
    info!("Getting audit trail for subject: {}", subject_id);

    // Get all pointers for subject
    let pointers = get_pointers_by_subject(&state.db_pool, &subject_id).await?;

    let active_count = pointers
        .iter()
        .filter(|p| matches!(p.status, PointerStatus::Active))
        .count();

    let orphaned_count = pointers
        .iter()
        .filter(|p| matches!(p.status, PointerStatus::Orphaned))
        .count();

    // Get audit trail
    let audit_logs = get_audit_trail_by_subject(&state.db_pool, &subject_id).await?;

    let audit_summaries: Vec<AuditEventSummary> = audit_logs
        .into_iter()
        .map(|log| AuditEventSummary {
            event_type: log.event_type,
            timestamp: log.timestamp.to_rfc3339(),
            pointer_id: log.pointer_id,
            event_data: log.event_data,
        })
        .collect();

    Ok(Json(GetAuditTrailResponse {
        subject_id,
        total_pointers: pointers.len(),
        active_pointers: active_count,
        orphaned_pointers: orphaned_count,
        audit_events: audit_summaries,
    }))
}
