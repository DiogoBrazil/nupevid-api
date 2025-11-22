pub struct VictimsQueries;

impl VictimsQueries {
    pub const CREATE_VICTIM: &'static str = r#"
        INSERT INTO victims (id, full_name, document_id, birth_date, phone, city_id)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
    "#;

    pub const GET_VICTIM_BY_ID: &'static str = r#"
        SELECT id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
        FROM victims
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_VICTIMS: &'static str = r#"
        SELECT id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
        FROM victims
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_VICTIMS_BY_CITY: &'static str = r#"
        SELECT id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
        FROM victims
        WHERE city_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const UPDATE_VICTIM_BY_ID: &'static str = r#"
        UPDATE victims
        SET full_name = $2, document_id = $3, birth_date = $4, phone = $5, city_id = $6
        WHERE id = $1 AND is_deleted = false
        RETURNING id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_BY_ID: &'static str = r#"
        UPDATE victims
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, full_name, document_id, birth_date, phone, city_id, created_at, updated_at, is_deleted
    "#;
}

pub struct VictimAddressesQueries;

impl VictimAddressesQueries {
    pub const CREATE_VICTIM_ADDRESS: &'static str = r#"
        INSERT INTO victim_addresses (id, victim_id, street, number, district, city_name, state, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, victim_id, street, number, district, city_name, state, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_VICTIM_ADDRESS_BY_VICTIM_ID: &'static str = r#"
        SELECT id, victim_id, street, number, district, city_name, state, zip_code, complement, created_at, updated_at, is_deleted
        FROM victim_addresses
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
        LIMIT 1
    "#;

    pub const UPDATE_VICTIM_ADDRESS_BY_VICTIM_ID: &'static str = r#"
        UPDATE victim_addresses
        SET street = $2, number = $3, district = $4, city_name = $5, state = $6, zip_code = $7, complement = $8
        WHERE victim_id = $1 AND is_deleted = false
        RETURNING id, victim_id, street, number, district, city_name, state, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_ADDRESS_BY_VICTIM_ID: &'static str = r#"
        UPDATE victim_addresses
        SET is_deleted = true
        WHERE victim_id = $1 AND is_deleted = false
        RETURNING id, victim_id, street, number, district, city_name, state, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_ADDRESS_EXISTS_FOR_VICTIM: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM victim_addresses
            WHERE victim_id = $1 AND is_deleted = false
        ) as exists
    "#;
}
