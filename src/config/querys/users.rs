pub struct UsersQueries;

impl UsersQueries {
    pub const CREATE_USER: &'static str = r#"
        INSERT INTO users (id, rank, registration, full_name, profile, email, password, city_id, permission_policies)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
    "#;

    pub const GET_USER_BY_ID: &'static str = r#"
        SELECT id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
        FROM users
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const UPDATE_USER_BY_ID: &'static str = r#"
        UPDATE users
        SET rank = $2, registration = $3, full_name = $4, profile = $5, email = $6, city_id = $7, permission_policies = $8
        WHERE id = $1 AND is_deleted = false
        RETURNING id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_USER_EXISTS_BY_EMAIL: &'static str = r#"
        SELECT EXISTS (SELECT 1 FROM users WHERE email = $1 AND is_deleted = false);
    "#;

    pub const CHECK_EMAIL_EXISTS_FOR_OTHER_USER: &'static str = r#"
        SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND id != $2 AND is_deleted = false) as exists
    "#;

    pub const GET_ALL_USERS: &'static str = r#"
        SELECT id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
        FROM users
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const DELETE_USER_BY_ID: &'static str = r#"
        UPDATE users
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
    "#;

    pub const GET_USER_PASSWORD_BY_ID: &'static str = r#"
        SELECT password FROM users WHERE id = $1 AND is_deleted = false
    "#;

    pub const UPDATE_USER_PASSWORD_BY_ID: &'static str = r#"
        UPDATE users
        SET password = $2
        WHERE id = $1 AND is_deleted = false
        RETURNING id, rank, registration, full_name, profile, email, city_id, permission_policies, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_CITY_ADMIN_EXISTS_FOR_CITY: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM users
            WHERE city_id = $1
            AND profile = 'CITY_ADMIN'
            AND is_deleted = false
            AND id != $2
        ) as exists
    "#;
}
