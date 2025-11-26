// Database models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "pointer_status", rename_all = "lowercase")]
pub enum PointerStatus {
    Active,
    Orphaned,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "receipt_operation", rename_all = "lowercase")]
pub enum ReceiptOperation {
    Create,
    Resolve,
    Orphan,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub org_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataStore {
    pub data_id: Uuid,
    pub org_id: Uuid,
    pub subject_id: String,
    pub content_hash: String,
    pub encrypted_payload: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Pointer {
    pub pointer_id: Uuid,
    pub org_id: Uuid,
    pub data_id: Uuid,
    pub subject_id: String,
    pub status: PointerStatus,
    pub created_at: DateTime<Utc>,
    pub orphaned_at: Option<DateTime<Utc>>,
    pub orphan_reason: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GovernanceReceipt {
    pub receipt_id: Uuid,
    pub pointer_id: Uuid,
    pub org_id: Uuid,
    pub operation: ReceiptOperation,
    pub receipt_json: serde_json::Value,
    pub receipt_hash: String,
    pub signature: Vec<u8>,
    pub signature_algorithm: String,
    pub prev_hash: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub log_id: Uuid,
    pub org_id: Option<Uuid>,
    pub pointer_id: Option<Uuid>,
    pub receipt_id: Option<Uuid>,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub actor_id: Option<String>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub timestamp: DateTime<Utc>,
}
