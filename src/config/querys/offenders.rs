pub struct OffendersQueries;

impl OffendersQueries {
    pub const CREATE_OFFENDER: &'static str = r#"
        INSERT INTO offenders (
            id, full_name, cpf, birth_date, city_id,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13::text[], $14, $15)
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
    "#;

    pub const GET_OFFENDER_BY_ID: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_ALL_OFFENDERS: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_PAGED: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, is_public_security_agent, security_force,
            uses_alcohol, uses_drugs, has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE is_deleted = false
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    "#;

    pub const GET_OFFENDERS_PAGED_BY_CITIES: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation, is_public_security_agent, security_force,
            uses_alcohol, uses_drugs, has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE is_deleted = false
        AND city_id = ANY($1)
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
    "#;

    pub const COUNT_OFFENDERS: &'static str = r#"
        SELECT COUNT(1)
        FROM offenders
        WHERE is_deleted = false
    "#;

    pub const COUNT_OFFENDERS_BY_CITIES: &'static str = r#"
        SELECT COUNT(1)
        FROM offenders
        WHERE is_deleted = false
        AND city_id = ANY($1)
    "#;

    pub const GET_OFFENDERS_BY_CITY: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE city_id = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_BY_NAME: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE full_name ILIKE $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_BY_CPF: &'static str = r#"
        SELECT
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
        FROM offenders
        WHERE cpf = $1 AND is_deleted = false
        ORDER BY created_at DESC
    "#;

    pub const GET_OFFENDERS_BY_VICTIM_ID: &'static str = r#"
        SELECT DISTINCT ON (o.id)
            o.id, o.full_name, o.cpf, o.birth_date, o.city_id, o.created_at, o.updated_at, o.is_deleted,
            o.imprisoned, o.occupation,
            o.is_public_security_agent, o.security_force,
            o.uses_alcohol, o.uses_drugs,
            o.has_psychiatric_issues, o.psychiatric_issues_type,
            o.education_level, o.observation
        FROM offenders o
        INNER JOIN protective_measures pm ON pm.offender_id = o.id AND pm.is_deleted = false
        WHERE pm.victim_id = $1 AND o.is_deleted = false
        ORDER BY o.id, o.created_at DESC
    "#;

    pub const UPDATE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE offenders
        SET
            full_name = $2, cpf = $3, birth_date = $4, city_id = $5,
            imprisoned = $6, occupation = $7,
            is_public_security_agent = $8, security_force = $9,
            uses_alcohol = $10, uses_drugs = $11,
            has_psychiatric_issues = $12, psychiatric_issues_type = $13::text[],
            education_level = $14, observation = $15
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
    "#;

    pub const DELETE_OFFENDER_BY_ID: &'static str = r#"
        UPDATE offenders
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING
            id, full_name, cpf, birth_date, city_id, created_at, updated_at, is_deleted,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation
    "#;
}

pub struct OffenderAddressesQueries;

impl OffenderAddressesQueries {
    pub const CREATE_OFFENDER_ADDRESS: &'static str = r#"
        INSERT INTO offender_addresses (id, offender_id, street, number, district, city_id, zip_code, complement, address_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const GET_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
        FROM offender_addresses
        WHERE id = $1 AND is_deleted = false
    "#;

    pub const GET_OFFENDER_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        SELECT id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
        FROM offender_addresses
        WHERE offender_id = $1 AND is_deleted = false
        ORDER BY created_at ASC
    "#;

    pub const UPDATE_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_addresses
        SET street = $2, number = $3, district = $4, city_id = $5, zip_code = $6, complement = $7, address_type = $8
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_ADDRESS_BY_ID: &'static str = r#"
        UPDATE offender_addresses
        SET is_deleted = true
        WHERE id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
    "#;

    pub const DELETE_OFFENDER_ADDRESSES_BY_OFFENDER_ID: &'static str = r#"
        UPDATE offender_addresses
        SET is_deleted = true
        WHERE offender_id = $1 AND is_deleted = false
        RETURNING id, offender_id, street, number, district, city_id, zip_code, complement, address_type, created_at, updated_at, is_deleted
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
