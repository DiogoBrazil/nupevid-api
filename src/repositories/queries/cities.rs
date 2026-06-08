pub struct CitiesQueries;

impl CitiesQueries {
    pub const CREATE_CITY: &'static str = r#"
        INSERT INTO cities (id, name, state, battalion)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, state, battalion, created_at, updated_at, is_deleted
    "#;

    pub const GET_CITY_BY_ID: &'static str = r#"
        SELECT id, name, state, battalion, created_at, updated_at, is_deleted
        FROM cities
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_CITIES: &'static str = r#"
        SELECT id, name, state, battalion, created_at, updated_at, is_deleted
        FROM cities
        WHERE is_deleted = false
        ORDER BY name ASC
    "#;

    pub const GET_CITIES_PAGED: &'static str = r#"
        SELECT id, name, state, battalion, created_at, updated_at, is_deleted
        FROM cities
        WHERE is_deleted = false
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_CITIES_PAGED_BY_IDS: &'static str = r#"
        SELECT id, name, state, battalion, created_at, updated_at, is_deleted
        FROM cities
        WHERE is_deleted = false
        AND id = ANY($1)
        ORDER BY name ASC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM cities
        WHERE is_deleted = false
    "#;

    pub const COUNT_CITIES_BY_IDS: &'static str = r#"
        SELECT COUNT(1)
        FROM cities
        WHERE is_deleted = false
        AND id = ANY($1)
    "#;

    pub const UPDATE_CITY_BY_ID: &'static str = r#"
        UPDATE cities
        SET name = $2, state = $3, battalion = $4
        WHERE id = $1 AND is_deleted = false
        RETURNING id, name, state, battalion, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_CITY_BY_ID: &'static str = r#"
        UPDATE cities
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, name, state, battalion, created_at, updated_at, is_deleted
    "#;

    pub const GET_CITY_BY_NAME_AND_BATTALION: &'static str = r#"
        SELECT id, name, state, battalion, created_at, updated_at, is_deleted
        FROM cities
        WHERE LOWER(name) = LOWER($1) AND LOWER(battalion) = LOWER($2) AND is_deleted = false
    "#;
}
