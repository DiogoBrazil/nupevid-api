pub struct OffendersQueries;

impl OffendersQueries {
    pub const CREATE_OFFENDER: &'static str = r#"
        INSERT INTO offenders (
            id, full_name, cpf, birth_date, city_id, victim_id,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
        RETURNING
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
    "#;

    pub const GET_OFFENDER_BY_ID: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
        FROM offenders
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_OFFENDERS: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
        FROM offenders
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_BY_CITY: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
        FROM offenders
        WHERE city_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_BY_VICTIM_ID: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
        FROM offenders
        WHERE victim_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const UPDATE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE offenders
        SET
            full_name = $2, cpf = $3, birth_date = $4, city_id = $5, victim_id = $6,
            imprisoned = $7, occupation = $8, workplace = $9,
            is_public_security_agent = $10, security_force = $11, relationship_to_victim = $12,
            uses_alcohol = $13, uses_drugs = $14,
            has_psychiatric_issues = $15, psychiatric_issues_type = $16,
            was_drunk_during_assault = $17, observation = $18
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
    "#;

    pub const DELETE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE offenders
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, victim_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, workplace,
            is_public_security_agent, security_force, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation
    "#;
}

pub struct OffenderAddressesQueries;

impl OffenderAddressesQueries {
    pub const CREATE_OFFENDER_ADDRESS: &'static str = r#"
        INSERT INTO offender_addresses (id, offender_id, street, number, district, city_id, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM offender_addresses
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_OFFENDER_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM offender_addresses
        WHERE offender_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_addresses
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        UPDATE offender_addresses
        SET is_deleted = true
        WHERE offender_id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;
}

pub struct OffenderPhonesQueries;

impl OffenderPhonesQueries {
    pub const CREATE_OFFENDER_PHONE: &'static str = r#"
        INSERT INTO offender_phones (id, offender_id, phone, phone_type)
        VALUES ($1, $2, $3, $4)
        RETURNING id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const GET_OFFENDER_PHONE_BY_ID: &'static str = r#"
        SELECT id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
        FROM offender_phones
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_OFFENDER_PHONES_BY_OFFENDER_ID: &'static str = r#"
        SELECT id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
        FROM offender_phones
        WHERE offender_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_OFFENDER_PHONE_BY_ID: &'static str = r#"
        UPDATE offender_phones
        SET phone = $2, phone_type = $3
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_PHONE_BY_ID: &'static str = r#"
        UPDATE offender_phones
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_PHONES_BY_OFFENDER_ID: &'static str = r#"
        UPDATE offender_phones
        SET is_deleted = true
        WHERE offender_id = $1 AND is_deleted = false
        RETURNING id, offender_id, phone, phone_type, created_at, updated_at, is_deleted
    "#;
}

pub struct OffenderWorkAddressesQueries;

impl OffenderWorkAddressesQueries {
    pub const CREATE_OFFENDER_WORK_ADDRESS: &'static str = r#"
        INSERT INTO offender_work_addresses (id, offender_id, street, number, district, city_id, zip_code, complement)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const GET_OFFENDER_WORK_ADDRESS_BY_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM offender_work_addresses
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_OFFENDER_WORK_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
        FROM offender_work_addresses
        WHERE offender_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_OFFENDER_WORK_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_work_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_WORK_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_work_addresses
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_WORK_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        UPDATE offender_work_addresses
        SET is_deleted = true
        WHERE offender_id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, created_at, updated_at, is_deleted
    "#;
}
