// Canonical receipt generation
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::{hashing::sha3_512_hash_str, Ed25519Keypair};
use crate::db::models::ReceiptOperation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptData {
    pub pointer_id: Uuid,
    pub operation: String,
    pub timestamp: DateTime<Utc>,
    pub subject_id: String,
    pub prev_hash: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedReceipt {
    pub receipt_json: serde_json::Value,
    pub receipt_hash: String,
    pub signature: Vec<u8>,
    pub signature_algorithm: String,
}

impl ReceiptData {
    pub fn new(
        pointer_id: Uuid,
        operation: ReceiptOperation,
        subject_id: String,
        prev_hash: Option<String>,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            pointer_id,
            operation: match operation {
                ReceiptOperation::Create => "create".to_string(),
                ReceiptOperation::Resolve => "resolve".to_string(),
                ReceiptOperation::Orphan => "orphan".to_string(),
            },
            timestamp: Utc::now(),
            subject_id,
            prev_hash,
            metadata,
        }
    }

    /// Convert to canonical JSON (sorted keys, no whitespace)
    pub fn to_canonical_json(&self) -> Result<String> {
        // Serialize with sorted keys for deterministic hashing
        let value = json!({
            "metadata": self.metadata,
            "operation": self.operation,
            "pointer_id": self.pointer_id,
            "prev_hash": self.prev_hash,
            "subject_id": self.subject_id,
            "timestamp": self.timestamp.to_rfc3339(),
        });

        // Compact JSON (no whitespace)
        Ok(serde_json::to_string(&value)?)
    }

    /// Generate signed receipt
    pub fn sign(&self, keypair: &Ed25519Keypair) -> Result<SignedReceipt> {
        // 1. Generate canonical JSON
        let canonical_json = self.to_canonical_json()?;

        // 2. Hash with SHA3-512
        let receipt_hash = sha3_512_hash_str(&canonical_json);

        // 3. Sign the hash with ED25519
        let signature = keypair.sign(receipt_hash.as_bytes());
        let signature_bytes = signature.to_bytes().to_vec();

        // 4. Return signed receipt
        Ok(SignedReceipt {
            receipt_json: serde_json::from_str(&canonical_json)?,
            receipt_hash,
            signature: signature_bytes,
            signature_algorithm: "ED25519".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_json_deterministic() {
        let receipt1 = ReceiptData::new(
            Uuid::new_v4(),
            ReceiptOperation::Create,
            "user_123".to_string(),
            None,
            json!({"test": "data"}),
        );

        let json1 = receipt1.to_canonical_json().unwrap();
        let json2 = receipt1.to_canonical_json().unwrap();

        assert_eq!(json1, json2, "Canonical JSON must be deterministic");
    }

    #[test]
    fn test_sign_receipt() {
        let keypair = Ed25519Keypair::generate();
        let receipt = ReceiptData::new(
            Uuid::new_v4(),
            ReceiptOperation::Create,
            "user_123".to_string(),
            None,
            json!({}),
        );

        let signed = receipt.sign(&keypair).unwrap();

        assert_eq!(signed.signature_algorithm, "ED25519");
        assert_eq!(signed.signature.len(), 64); // ED25519 signature is 64 bytes
        assert!(!signed.receipt_hash.is_empty());
    }
}
