pub struct AttendanceOffendersQueries;

impl AttendanceOffendersQueries {
    pub const CREATE_ATTENDANCE_OFFENDER: &'static str = r#"
        INSERT INTO attendance_offenders (
            id, offender_id, victim_id, protective_measure_id, was_offender_present,
            attendance_date, attendance_time, is_remote, assaults_children,
            violence_aggravator, violence_aggravator_other, description
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING id, offender_id, victim_id, protective_measure_id, was_offender_present,
                  attendance_date, attendance_time, is_remote, assaults_children,
                  violence_aggravator, violence_aggravator_other, description,
                  created_at, updated_at, is_deleted
    "#;

    pub const GET_ATTENDANCE_OFFENDER_BY_ID: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_ATTENDANCE_OFFENDERS: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_PAGED: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_PAGED_BY_CITIES: &'static str = r#"
        SELECT ao.id, ao.offender_id, ao.victim_id, ao.protective_measure_id, ao.was_offender_present,
               ao.attendance_date, ao.attendance_time, ao.is_remote, ao.assaults_children,
               ao.violence_aggravator, ao.violence_aggravator_other, ao.description,
               ao.created_at, ao.updated_at, ao.is_deleted
        FROM attendance_offenders ao
        JOIN offenders o ON o.id = ao.offender_id
        WHERE ao.is_deleted = false
        AND o.city_id = ANY($1)
        ORDER BY ao.attendance_date DESC, ao.attendance_time DESC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_ATTENDANCE_OFFENDERS: &'static str = r#"
        SELECT COUNT(1)
        FROM attendance_offenders
        WHERE is_deleted = false
    "#;

    pub const COUNT_ATTENDANCE_OFFENDERS_BY_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM attendance_offenders ao
        JOIN offenders o ON o.id = ao.offender_id
        WHERE ao.is_deleted = false
        AND o.city_id = ANY($1)
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_BY_OFFENDER: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE offender_id = $1 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_BY_OFFENDER_AND_MEASURE: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE offender_id = $1 AND protective_measure_id = $2 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_BY_VICTIM: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_OFFENDERS_BY_VICTIM_AND_MEASURE: &'static str = r#"
        SELECT id, offender_id, victim_id, protective_measure_id, was_offender_present,
               attendance_date, attendance_time, is_remote, assaults_children,
               violence_aggravator, violence_aggravator_other, description,
               created_at, updated_at, is_deleted
        FROM attendance_offenders
        WHERE victim_id = $1 AND protective_measure_id = $2 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const UPDATE_ATTENDANCE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE attendance_offenders
        SET offender_id = $2, victim_id = $3, protective_measure_id = $4,
            was_offender_present = $5, attendance_date = $6, attendance_time = $7,
            is_remote = $8, assaults_children = $9, violence_aggravator = $10,
            violence_aggravator_other = $11, description = $12
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, victim_id, protective_measure_id, was_offender_present,
                  attendance_date, attendance_time, is_remote, assaults_children,
                  violence_aggravator, violence_aggravator_other, description,
                  created_at, updated_at, is_deleted
    "#;

    pub const DELETE_ATTENDANCE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE attendance_offenders
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, victim_id, protective_measure_id, was_offender_present,
                  attendance_date, attendance_time, is_remote, assaults_children,
                  violence_aggravator, violence_aggravator_other, description,
                  created_at, updated_at, is_deleted
    "#;
}

pub struct AttendanceOffenderAddressesQueries;

impl AttendanceOffenderAddressesQueries {
    pub const CREATE_ATTENDANCE_OFFENDER_ADDRESS: &'static str = r#"
        INSERT INTO attendance_offender_addresses (id, attendance_id, street, number, district, city_id, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        SELECT id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM attendance_offender_addresses
        WHERE attendance_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        LIMIT 1
    "#;

    pub const UPDATE_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_offender_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_ATTENDANCE_OFFENDER_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_offender_addresses
        SET is_deleted = true
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE_OFFENDER: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM attendance_offender_addresses
            WHERE attendance_id = $1 AND is_deleted = false
        ) as exists
    "#;
}
