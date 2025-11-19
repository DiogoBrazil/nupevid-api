pub struct AuthQueries;

impl AuthQueries {
    pub const GET_COMPLETE_USER_DATA_BY_EMAIL: &'static str = r#"
    SELECT id, rank, registration, full_name, profile, email, password, created_at, updated_at
    FROM users
    WHERE email = $1
    "#;
}
