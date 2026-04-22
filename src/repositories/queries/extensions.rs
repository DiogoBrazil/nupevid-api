pub struct ExtensionsQueries;

impl ExtensionsQueries {
    pub const CREATE_EXTENSION: &'static str = r#"
        INSERT INTO protective_measure_extensions (id, protective_measure_id, extension_number, extension_date, new_valid_until, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
    "#;

    pub const GET_EXTENSION_BY_ID: &'static str = r#"
        SELECT id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
        FROM protective_measure_extensions
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_EXTENSIONS_BY_MEASURE: &'static str = r#"
        SELECT id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
        FROM protective_measure_extensions
        WHERE protective_measure_id = $1 AND is_deleted = false
        ORDER BY extension_number ASC
    "#;

    pub const GET_ALL_EXTENSIONS: &'static str = r#"
        SELECT id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
        FROM protective_measure_extensions
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const UPDATE_EXTENSION_BY_ID: &'static str = r#"
        UPDATE protective_measure_extensions
        SET extension_number = $2, extension_date = $3, new_valid_until = $4, notes = $5
        WHERE id = $1 AND is_deleted = false
        RETURNING id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_EXTENSION_BY_ID: &'static str = r#"
        UPDATE protective_measure_extensions
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, protective_measure_id, extension_number, extension_date, new_valid_until, notes, created_at, updated_at, is_deleted
    "#;
}
