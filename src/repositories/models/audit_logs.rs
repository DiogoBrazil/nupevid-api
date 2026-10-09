use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

use crate::core::entities::audit_logs::AuditLogEntry;

#[derive(sqlx::FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub request_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<Uuid>,
    pub city_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: JsonValue,
}

impl From<AuditLogRow> for AuditLogEntry {
    fn from(row: AuditLogRow) -> Self {
        Self {
            id: row.id,
            occurred_at: row.occurred_at,
            request_id: row.request_id,
            user_id: row.user_id,
            action: row.action,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            city_id: row.city_id,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            details: row.details,
        }
    }
}
