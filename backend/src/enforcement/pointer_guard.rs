// Pointer orphaning enforcement layer
// Per US 19/240,581 Claim 9: Orphaned pointers cannot be resolved

use crate::db::models::{Pointer, PointerStatus};
use anyhow::{anyhow, Result};

/// Enforces pointer access rules
/// Returns Err if pointer is orphaned
pub fn enforce_pointer_access(pointer: &Pointer) -> Result<()> {
    match pointer.status {
        PointerStatus::Active => Ok(()),
        PointerStatus::Orphaned => Err(anyhow!(
            "pointer_orphaned: This pointer has been orphaned and cannot be resolved"
        )),
    }
}

/// Check if pointer can be accessed
pub fn is_pointer_accessible(pointer: &Pointer) -> bool {
    matches!(pointer.status, PointerStatus::Active)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_pointer(status: PointerStatus) -> Pointer {
        Pointer {
            pointer_id: Uuid::new_v4(),
            org_id: Uuid::new_v4(),
            data_id: Uuid::new_v4(),
            subject_id: "test_user".to_string(),
            status,
            created_at: Utc::now(),
            orphaned_at: None,
            orphan_reason: None,
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_active_pointer_accessible() {
        let pointer = create_test_pointer(PointerStatus::Active);
        assert!(enforce_pointer_access(&pointer).is_ok());
        assert!(is_pointer_accessible(&pointer));
    }

    #[test]
    fn test_orphaned_pointer_blocked() {
        let pointer = create_test_pointer(PointerStatus::Orphaned);
        assert!(enforce_pointer_access(&pointer).is_err());
        assert!(!is_pointer_accessible(&pointer));
    }
}
