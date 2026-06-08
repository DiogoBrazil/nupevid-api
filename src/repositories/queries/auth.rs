pub struct AuthQueries;

impl AuthQueries {
    pub const GET_COMPLETE_USER_DATA_BY_EMAIL: &'static str = r#"
    SELECT id, rank, registration, full_name, profile, email, password, city_id,
           is_temporary_password, temporary_password_expires_at,
           created_at, updated_at, is_deleted
    FROM users
    WHERE email = $1 AND is_deleted = false
    "#;
}
