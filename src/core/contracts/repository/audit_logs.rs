use async_trait::async_trait;

use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::audit_logs::{AuditLogEntry, NewAuditLogEntry};

#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn create_audit_log(
        &self,
        entry: NewAuditLogEntry,
    ) -> Result<AuditLogEntry, RepositoryError>;
}
