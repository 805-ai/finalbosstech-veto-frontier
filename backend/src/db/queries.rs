// Database queries
use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;

// ============================================================================
// DATA STORE QUERIES
// ============================================================================

pub async fn create_data_store(
    pool: &PgPool,
    org_id: Uuid,
    subject_id: &str,
    content_hash: &str,
    encrypted_payload: Option<&[u8]>,
) -> Result<DataStore> {
    let data = sqlx::query_as::<_, DataStore>(
        r#"
        INSERT INTO data_store (org_id, subject_id, content_hash, encrypted_payload)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(subject_id)
    .bind(content_hash)
    .bind(encrypted_payload)
    .fetch_one(pool)
    .await
    .context("Failed to insert into data_store")?;

    Ok(data)
}

pub async fn get_data_store(pool: &PgPool, data_id: Uuid) -> Result<Option<DataStore>> {
    let data = sqlx::query_as::<_, DataStore>(
        r#"
        SELECT * FROM data_store WHERE data_id = $1
        "#,
    )
    .bind(data_id)
    .fetch_optional(pool)
    .await
    .context("Failed to query data_store")?;

    Ok(data)
}

// ============================================================================
// POINTER QUERIES
// ============================================================================

pub async fn create_pointer(
    pool: &PgPool,
    org_id: Uuid,
    data_id: Uuid,
    subject_id: &str,
) -> Result<Pointer> {
    let pointer = sqlx::query_as::<_, Pointer>(
        r#"
        INSERT INTO pointers (org_id, data_id, subject_id, status)
        VALUES ($1, $2, $3, 'active')
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(data_id)
    .bind(subject_id)
    .fetch_one(pool)
    .await
    .context("Failed to insert pointer")?;

    Ok(pointer)
}

pub async fn get_pointer(pool: &PgPool, pointer_id: Uuid) -> Result<Option<Pointer>> {
    let pointer = sqlx::query_as::<_, Pointer>(
        r#"
        SELECT * FROM pointers WHERE pointer_id = $1
        "#,
    )
    .bind(pointer_id)
    .fetch_optional(pool)
    .await
    .context("Failed to query pointer")?;

    Ok(pointer)
}

pub async fn orphan_pointer(
    pool: &PgPool,
    pointer_id: Uuid,
    reason: Option<&str>,
) -> Result<Pointer> {
    let pointer = sqlx::query_as::<_, Pointer>(
        r#"
        UPDATE pointers
        SET status = 'orphaned',
            orphaned_at = NOW(),
            orphan_reason = $2
        WHERE pointer_id = $1
        RETURNING *
        "#,
    )
    .bind(pointer_id)
    .bind(reason)
    .fetch_one(pool)
    .await
    .context("Failed to orphan pointer")?;

    Ok(pointer)
}

pub async fn get_pointers_by_subject(
    pool: &PgPool,
    subject_id: &str,
) -> Result<Vec<Pointer>> {
    let pointers = sqlx::query_as::<_, Pointer>(
        r#"
        SELECT * FROM pointers
        WHERE subject_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(subject_id)
    .fetch_all(pool)
    .await
    .context("Failed to query pointers by subject")?;

    Ok(pointers)
}

// ============================================================================
// GOVERNANCE RECEIPT QUERIES
// ============================================================================

pub async fn create_governance_receipt(
    pool: &PgPool,
    pointer_id: Uuid,
    org_id: Uuid,
    operation: ReceiptOperation,
    receipt_json: serde_json::Value,
    receipt_hash: &str,
    signature: &[u8],
    signature_algorithm: &str,
    prev_hash: Option<&str>,
) -> Result<GovernanceReceipt> {
    let receipt = sqlx::query_as::<_, GovernanceReceipt>(
        r#"
        INSERT INTO governance_receipts
            (pointer_id, org_id, operation, receipt_json, receipt_hash,
             signature, signature_algorithm, prev_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(pointer_id)
    .bind(org_id)
    .bind(operation)
    .bind(receipt_json)
    .bind(receipt_hash)
    .bind(signature)
    .bind(signature_algorithm)
    .bind(prev_hash)
    .fetch_one(pool)
    .await
    .context("Failed to insert governance receipt")?;

    Ok(receipt)
}

pub async fn get_receipts_by_pointer(
    pool: &PgPool,
    pointer_id: Uuid,
) -> Result<Vec<GovernanceReceipt>> {
    let receipts = sqlx::query_as::<_, GovernanceReceipt>(
        r#"
        SELECT * FROM governance_receipts
        WHERE pointer_id = $1
        ORDER BY timestamp ASC
        "#,
    )
    .bind(pointer_id)
    .fetch_all(pool)
    .await
    .context("Failed to query receipts by pointer")?;

    Ok(receipts)
}

pub async fn get_latest_receipt_hash(
    pool: &PgPool,
    pointer_id: Uuid,
) -> Result<Option<String>> {
    let result = sqlx::query_scalar::<_, Option<String>>(
        r#"
        SELECT receipt_hash FROM governance_receipts
        WHERE pointer_id = $1
        ORDER BY timestamp DESC
        LIMIT 1
        "#,
    )
    .bind(pointer_id)
    .fetch_one(pool)
    .await
    .context("Failed to query latest receipt hash")?;

    Ok(result)
}

// ============================================================================
// AUDIT LOG QUERIES
// ============================================================================

pub async fn create_audit_log(
    pool: &PgPool,
    org_id: Option<Uuid>,
    pointer_id: Option<Uuid>,
    receipt_id: Option<Uuid>,
    event_type: &str,
    event_data: serde_json::Value,
    actor_id: Option<&str>,
) -> Result<AuditLog> {
    let log = sqlx::query_as::<_, AuditLog>(
        r#"
        INSERT INTO audit_log
            (org_id, pointer_id, receipt_id, event_type, event_data, actor_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(org_id)
    .bind(pointer_id)
    .bind(receipt_id)
    .bind(event_type)
    .bind(event_data)
    .bind(actor_id)
    .fetch_one(pool)
    .await
    .context("Failed to insert audit log")?;

    Ok(log)
}

pub async fn get_audit_trail_by_subject(
    pool: &PgPool,
    subject_id: &str,
) -> Result<Vec<AuditLog>> {
    let logs = sqlx::query_as::<_, AuditLog>(
        r#"
        SELECT al.* FROM audit_log al
        JOIN pointers p ON al.pointer_id = p.pointer_id
        WHERE p.subject_id = $1
        ORDER BY al.timestamp DESC
        "#,
    )
    .bind(subject_id)
    .fetch_all(pool)
    .await
    .context("Failed to query audit trail by subject")?;

    Ok(logs)
}
