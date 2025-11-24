pub struct AttendancesQueries;

impl AttendancesQueries {
    pub const CREATE_ATTENDANCE: &'static str = r#"
        INSERT INTO attendances (id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
    "#;

    pub const GET_ATTENDANCE_BY_ID: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
        FROM attendances
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_ATTENDANCES: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
        FROM attendances
        WHERE is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const GET_ATTENDANCES_BY_VICTIM: &'static str = r#"
        SELECT id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
        FROM attendances
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY attendance_date DESC, attendance_time DESC
    "#;

    pub const UPDATE_ATTENDANCE_BY_ID: &'static str = r#"
        UPDATE attendances
        SET victim_id = $2, was_victim_present = $3, attendance_date = $4, attendance_time = $5, description = $6, latitude = $7, longitude = $8
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_ATTENDANCE_BY_ID: &'static str = r#"
        UPDATE attendances
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, was_victim_present, attendance_date, attendance_time, description, latitude, longitude, created_at, updated_at, is_deleted
    "#;
}

pub struct AttendanceAddressesQueries;

impl AttendanceAddressesQueries {
    pub const CREATE_ATTENDANCE_ADDRESS: &'static str = r#"
        INSERT INTO attendance_addresses (id, attendance_id, street, number, district, city_id, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        SELECT id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM attendance_addresses
        WHERE attendance_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        LIMIT 1
    "#;

    pub const UPDATE_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_ATTENDANCE_ADDRESS_BY_ATTENDANCE_ID: &'static str = r#"
        UPDATE attendance_addresses
        SET is_deleted = true
        WHERE attendance_id = $1 AND is_deleted = false
        RETURNING id, attendance_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_ADDRESS_EXISTS_FOR_ATTENDANCE: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM attendance_addresses
            WHERE attendance_id = $1 AND is_deleted = false
        ) as exists
    "#;
}
