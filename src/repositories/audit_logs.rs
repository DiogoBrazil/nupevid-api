use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::contracts::repository::audit_logs::AuditLogRepository;
use crate::core::contracts::repository::error::RepositoryError;
use crate::core::entities::audit_logs::{AuditLogEntry, NewAuditLogEntry};
use crate::repositories::error_mapper::map_sqlx_error;
use crate::repositories::models::audit_logs::AuditLogRow;
use crate::repositories::queries::audit_logs::AuditLogQueries;

#[derive(Clone)]
pub struct PgAuditLogRepository {
    pool: PgPool,
}

impl PgAuditLogRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuditLogRepository for PgAuditLogRepository {
    async fn create_audit_log(
        &self,
        entry: NewAuditLogEntry,
    ) -> Result<AuditLogEntry, RepositoryError> {
        let row: AuditLogEntry =
            sqlx::query_as::<_, AuditLogRow>(AuditLogQueries::CREATE_AUDIT_LOG)
                .bind(Uuid::new_v4())
                .bind(entry.request_id)
                .bind(entry.user_id)
                .bind(&entry.action)
                .bind(&entry.entity_type)
                .bind(entry.entity_id)
                .bind(entry.city_id)
                .bind(&entry.ip_address)
                .bind(&entry.user_agent)
                .bind(entry.details)
                .fetch_one(&self.pool)
                .await
                .map_err(map_sqlx_error)?
                .into();
        Ok(row)
    }
}
