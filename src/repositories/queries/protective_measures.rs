pub struct ProtectiveMeasuresQueries;

impl ProtectiveMeasuresQueries {
    pub const CREATE_PROTECTIVE_MEASURE: &'static str = r#"
        INSERT INTO protective_measures (
            id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until,
            judicial_authority, court_district_id, distance_meters, status, violence_types,
            relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        RETURNING id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
    "#;

    pub const GET_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        SELECT id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_PROTECTIVE_MEASURES: &'static str = r#"
        SELECT id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_PROTECTIVE_MEASURES_PAGED: &'static str = r#"
        SELECT id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE is_deleted = false
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_PROTECTIVE_MEASURES_PAGED_BY_CITIES: &'static str = r#"
        SELECT pm.id, pm.process_number, pm.sei_process_number, pm.occurrence_report_number, pm.issued_at, pm.valid_until, pm.judicial_authority, pm.court_district_id, pm.distance_meters, pm.status, pm.violence_types, pm.relationship_to_victim, pm.assaults_children, pm.was_drunk_during_assault, pm.victim_id, pm.offender_id, pm.created_at, pm.updated_at, pm.is_deleted
        FROM protective_measures pm
        JOIN victims v ON v.id = pm.victim_id
        WHERE pm.is_deleted = false
        AND v.city_id = ANY($1)
        ORDER BY pm.created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_PROTECTIVE_MEASURES: &'static str = r#"
        SELECT COUNT(1)
        FROM protective_measures
        WHERE is_deleted = false
    "#;

    pub const COUNT_PROTECTIVE_MEASURES_BY_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM protective_measures pm
        JOIN victims v ON v.id = pm.victim_id
        WHERE pm.is_deleted = false
        AND v.city_id = ANY($1)
    "#;

    pub const GET_PROTECTIVE_MEASURES_BY_VICTIM: &'static str = r#"
        SELECT id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
        FROM protective_measures
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const CHECK_ACTIVE_MEASURE_EXISTS_FOR_VICTIM: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM protective_measures
            WHERE victim_id = $1
            AND status = 'Valid'
            AND is_deleted = false
            AND id != $2
        ) as exists
    "#;

    pub const UPDATE_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        UPDATE protective_measures
        SET process_number = $2, sei_process_number = $3, occurrence_report_number = $4, issued_at = $5, valid_until = $6, judicial_authority = $7, court_district_id = $8, distance_meters = $9, status = $10, violence_types = $11, relationship_to_victim = $12, assaults_children = $13, was_drunk_during_assault = $14, victim_id = $15, offender_id = $16
        WHERE id = $1 AND is_deleted = false
        RETURNING id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_PROTECTIVE_MEASURE_BY_ID: &'static str = r#"
        UPDATE protective_measures
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, process_number, sei_process_number, occurrence_report_number, issued_at, valid_until, judicial_authority, court_district_id, distance_meters, status, violence_types, relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id, created_at, updated_at, is_deleted
    "#;
}
