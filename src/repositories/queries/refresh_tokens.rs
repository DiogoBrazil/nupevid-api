pub struct RefreshTokenQueries;

impl RefreshTokenQueries {
    pub const CREATE_REFRESH_TOKEN: &'static str = r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, user_agent, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, user_id, token_hash, expires_at, revoked_at, replaced_by_token_id,
                  created_at, user_agent, ip_address
    "#;

    pub const GET_REFRESH_TOKEN_BY_ID: &'static str = r#"
        SELECT id, user_id, token_hash, expires_at, revoked_at, replaced_by_token_id,
               created_at, user_agent, ip_address
        FROM refresh_tokens
        WHERE id = $1
    "#;

    pub const REVOKE_AND_REPLACE_REFRESH_TOKEN: &'static str = r#"
        UPDATE refresh_tokens
        SET revoked_at = NOW(), replaced_by_token_id = $2
        WHERE id = $1 AND revoked_at IS NULL
    "#;

    pub const REVOKE_REFRESH_TOKEN: &'static str = r#"
        UPDATE refresh_tokens
        SET revoked_at = NOW()
        WHERE id = $1 AND revoked_at IS NULL
    "#;
}
