pub struct AttendanceVictimsQueries;

impl AttendanceVictimsQueries {
    pub const CREATE_ATTENDANCE_VICTIM: &'static str = r#"
        INSERT INTO attendance_victims (
            id, victim_id, was_victim_present, attendance_date, attendance_time, description,
            latitude, longitude, offender_id, protective_measure_id, is_remote, risk_level,
            offender_freedom_status, offender_has_firearm_access, needs_legal_assistance,
            needs_psychological_support, was_instructed_about_protective_measure_procedures,
            offender_violated_protective_measure
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description,
                  latitude, longitude, created_at, updated_at, is_deleted, offender_id,
                  protective_measure_id, is_remote, risk_level, offender_freedom_status,
                  offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
                  was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
    "#;

    pub const GET_ATTENDANCE_VICTIM_BY_ID: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description,
               latitude, longitude, created_at, updated_at, is_deleted, offender_id,
               protective_measure_id, is_remote, risk_level, offender_freedom_status,
               offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
               was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
        FROM attendance_victims
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_ATTENDANCE_VICTIMS: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description,
               latitude, longitude, created_at, updated_at, is_deleted, offender_id,
               protective_measure_id, is_remote, risk_level, offender_freedom_status,
               offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
               was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
        FROM attendance_victims
        WHERE is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_VICTIMS_PAGED: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description,
               latitude, longitude, created_at, updated_at, is_deleted, offender_id,
               protective_measure_id, is_remote, risk_level, offender_freedom_status,
               offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
               was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
        FROM attendance_victims
        WHERE is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_ATTENDANCE_VICTIMS_PAGED_BY_CITIES: &'static str = r#"
        SELECT av.id, av.victim_id, av.was_victim_present, av.attendance_date, av.attendance_time, av.description,
               av.latitude, av.longitude, av.created_at, av.updated_at, av.is_deleted, av.offender_id,
               av.protective_measure_id, av.is_remote, av.risk_level, av.offender_freedom_status,
               av.offender_has_firearm_access, av.needs_legal_assistance, av.needs_psychological_support,
               av.was_instructed_about_protective_measure_procedures, av.offender_violated_protective_measure
        FROM attendance_victims av
        JOIN victims v ON v.id = av.victim_id
        WHERE av.is_deleted = false
        AND v.city_id = ANY($1)
        ORDER BY av.attendance_date DESC, av.attendance_time DESC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_ATTENDANCE_VICTIMS: &'static str = r#"
        SELECT COUNT(1)
        FROM attendance_victims
        WHERE is_deleted = false
    "#;

    pub const COUNT_ATTENDANCE_VICTIMS_BY_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM attendance_victims av
        JOIN victims v ON v.id = av.victim_id
        WHERE av.is_deleted = false
        AND v.city_id = ANY($1)
    "#;

    pub const GET_ATTENDANCE_VICTIMS_BY_VICTIM: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description,
               latitude, longitude, created_at, updated_at, is_deleted, offender_id,
               protective_measure_id, is_remote, risk_level, offender_freedom_status,
               offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
               was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
        FROM attendance_victims
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCE_VICTIMS_BY_MEASURE: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description,
               latitude, longitude, created_at, updated_at, is_deleted, offender_id,
               protective_measure_id, is_remote, risk_level, offender_freedom_status,
               offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
               was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
        FROM attendance_victims
        WHERE protective_measure_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_ATTENDANCE_VICTIM_BY_ID: &'static str = r#"
        UPDATE attendance_victims
        SET victim_id = $2, was_victim_present = $3, attendance_date = $4, attendance_time = $5,
            description = $6, latitude = $7, longitude = $8, offender_id = $9,
            protective_measure_id = $10, is_remote = $11, risk_level = $12,
            offender_freedom_status = $13, offender_has_firearm_access = $14,
            needs_legal_assistance = $15, needs_psychological_support = $16,
            was_instructed_about_protective_measure_procedures = $17,
            offender_violated_protective_measure = $18
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description,
                  latitude, longitude, created_at, updated_at, is_deleted, offender_id,
                  protective_measure_id, is_remote, risk_level, offender_freedom_status,
                  offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
                  was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
    "#;

    pub const DELETE_ATTENDANCE_VICTIM_BY_ID: &'static str = r#"
        UPDATE attendance_victims
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description,
                  latitude, longitude, created_at, updated_at, is_deleted, offender_id,
                  protective_measure_id, is_remote, risk_level, offender_freedom_status,
                  offender_has_firearm_access, needs_legal_assistance, needs_psychological_support,
                  was_instructed_about_protective_measure_procedures, offender_violated_protective_measure
    "#;
}

pub struct AttendanceVictimAddressesQueries;

impl AttendanceVictimAddressesQueries {
    pub const CREATE_ATTENDANCE_VICTIM_ADDRESS: &'static str = r#"
        INSERT INTO attendance_victim_addresses (id, attendance_id, street, number, district, city_id, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        SELECT id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM attendance_victim_addresses
        WHERE attendance_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        LIMIT 1
    "#;

    pub const UPDATE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_victim_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_ATTENDANCE_VICTIM_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_victim_addresses
        SET is_deleted = true
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE_VICTIM: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM attendance_victim_addresses
            WHERE attendance_id = $1 AND is_deleted = false
        ) as exists
    "#;
}
