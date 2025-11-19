pub struct UsersQueries;

impl UsersQueries {
    pub const CREATE_USER: &'static str = r#"
        INSERT INTO users (id, rank, registration, full_name, profile, email, password)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, rank, registration, full_name, profile, email, created_at, updated_at
    "#;
}
