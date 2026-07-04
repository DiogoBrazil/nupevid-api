pub struct AuditLogQueries;

impl AuditLogQueries {
    pub const CREATE_AUDIT_LOG: &'static str = r#"
        INSERT INTO audit_log (
            id, request_id, user_id, action, entity_type, entity_id,
            city_id, ip_address, user_agent, details
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, occurred_at, request_id, user_id, action, entity_type,
                  entity_id, city_id, ip_address, user_agent, details
    "#;
}
