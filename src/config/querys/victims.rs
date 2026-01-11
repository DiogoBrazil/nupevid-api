pub struct VictimsQueries;

impl VictimsQueries {
    pub const CREATE_VICTIM: &'static str = r#"
        INSERT INTO victims (
            id, full_name, cpf, birth_date, city_id,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::text[], $13, $14, $15, $16::text[])
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
    "#;

    pub const GET_VICTIM_BY_ID: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_VICTIMS: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_VICTIMS_PAGED: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE is_deleted = false
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_VICTIMS_PAGED_BY_CITIES: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE is_deleted = false
        AND city_id = ANY($1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_VICTIMS: &'static str = r#"
        SELECT COUNT(1)
        FROM victims
        WHERE is_deleted = false
    "#;

    pub const COUNT_VICTIMS_BY_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM victims
        WHERE is_deleted = false
        AND city_id = ANY($1)
    "#;

    pub const GET_VICTIMS_BY_CITY: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE city_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_VICTIMS_BY_NAME: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE full_name ILIKE $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_VICTIMS_BY_CPF: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
        FROM victims
        WHERE cpf = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const UPDATE_VICTIM_BY_ID: &'static str = r#"
        UPDATE victims
        SET
            full_name = $2, cpf = $3, birth_date = $4, city_id = $5,
            education_level = $6, occupation = $7,
            has_children = $8, children_count = $9, is_pregnant = $10,
            has_special_needs = $11, special_needs_type = $12::text[],
            uses_alcohol = $13, uses_drugs = $14,
            has_psychiatric_issues = $15, psychiatric_issues_type = $16::text[]
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
    "#;

    pub const DELETE_VICTIM_BY_ID: &'static str = r#"
        UPDATE victims
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            education_level, occupation,
            has_children, children_count, is_pregnant,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type
    "#;
}

pub struct VictimAddressesQueries;

impl VictimAddressesQueries {
    pub const CREATE_VICTIM_ADDRESS: &'static str = r#"
        INSERT INTO victim_addresses (id, victim_id, street, number, district, city_id, zip_code, complement, address_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const GET_VICTIM_ADDRESS_BY_ID: &'static str = r#"
        SELECT id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
        FROM victim_addresses
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_VICTIM_ADDRESSES_BY_VICTIM_ID: &'static str = r#"
        SELECT id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
        FROM victim_addresses
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_VICTIM_ADDRESS_BY_ID: &'static str = r#"
        UPDATE victim_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7, address_type = $8
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_ADDRESS_BY_ID: &'static str = r#"
        UPDATE victim_addresses
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_ADDRESSES_BY_VICTIM_ID: &'static str = r#"
        UPDATE victim_addresses
        SET is_deleted = true
        WHERE victim_id = $1 AND is_deleted = false
        RETURNING id, victim_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const CHECK_ADDRESS_EXISTS_FOR_VICTIM: &'static str = r#"
        SELECT EXISTS(
            SELECT 1 FROM victim_addresses
            WHERE victim_id = $1 AND is_deleted = false
        ) as exists
    "#;
}

pub struct VictimPhonesQueries;

impl VictimPhonesQueries {
    pub const CREATE_VICTIM_PHONE: &'static str = r#"
        INSERT INTO victim_phones (id, victim_id, phone, phone_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const GET_VICTIM_PHONE_BY_ID: &'static str = r#"
        SELECT id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
        FROM victim_phones
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_VICTIM_PHONES_BY_VICTIM_ID: &'static str = r#"
        SELECT id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
        FROM victim_phones
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_VICTIM_PHONE_BY_ID: &'static str = r#"
        UPDATE victim_phones
        SET phone = $2, phone_type = $3
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_PHONE_BY_ID: &'static str = r#"
        UPDATE victim_phones
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_VICTIM_PHONES_BY_VICTIM_ID: &'static str = r#"
        UPDATE victim_phones
        SET is_deleted = true
        WHERE victim_id = $1 AND is_deleted = false
        RETURNING id, victim_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;
}
