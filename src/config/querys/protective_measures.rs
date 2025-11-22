pub struct ProtectiveMeasuresQueries;

impl ProtectiveMeasuresQueries {
    pub const CREATE_PROTECTIVE_MEASURE: &'static str = r#"
        INSERT INTO protective_measures (id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
    "#;

    pub const GET_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        SELECT id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_PROTECTIVE_MEASURES: &'static str = r#"
        SELECT id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_PROTECTIVE_MEASURES_BY_VICTIM: &'static str = r#"
        SELECT id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const CHECK_ACTIVE_MEASURE_EXISTS_FOR_VICTIM: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM protective_measures
            WHERE victim_id = $1
            AND is_active = true
            AND is_deleted = false
            AND id != $2
        ) as exists
    "#;

    pub const UPDATE_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        UPDATE protective_measures
        SET process_number = $2, issued_at = $3, judicial_authority = $4, court_district = $5, is_active = $6, victim_id = $7
        WHERE id = $1 AND is_deleted = false
        RETURNING id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        UPDATE protective_measures
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, process_number, issued_at, judicial_authority, court_district, is_active, victim_id, created_at, updated_at, is_deleted
    "#;
}
