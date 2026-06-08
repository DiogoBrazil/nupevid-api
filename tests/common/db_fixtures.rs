use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

/// Insert a test city into the database and return its id.
pub async fn insert_city(pool: &PgPool, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO cities (id, name, state, battalion, is_deleted) \
         VALUES ($1, $2, $3, $4, false)",
    )
    .bind(id)
    .bind(name)
    .bind("RO")
    .bind("1ºBPM")
    .execute(pool)
    .await
    .expect("Failed to insert test city");
    id
}

/// Insert a test victim associated with the given city and return its id.
pub async fn insert_victim(pool: &PgPool, full_name: &str, city_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO victims (
            id, full_name, cpf, birth_date, city_id,
            education_level, occupation,
            has_children, children_count,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6::education_level_enum, $7, $8, $9, $10, $11, $12, $13, $14, $15, false
        )",
    )
    .bind(id)
    .bind(full_name)
    .bind(Option::<String>::None) // cpf
    .bind(Option::<NaiveDate>::None) // birth_date
    .bind(city_id)
    .bind(Option::<String>::None) // education_level
    .bind(Option::<String>::None) // occupation
    .bind(false) // has_children
    .bind(Option::<i32>::None) // children_count
    .bind(false) // has_special_needs
    .bind(Option::<Vec<String>>::None) // special_needs_type
    .bind(false) // uses_alcohol
    .bind(false) // uses_drugs
    .bind(false) // has_psychiatric_issues
    .bind(Option::<Vec<String>>::None) // psychiatric_issues_type
    .execute(pool)
    .await
    .expect("Failed to insert test victim");
    id
}

/// Insert a test offender associated with the given city, return its id.
pub async fn insert_offender(pool: &PgPool, full_name: &str, city_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO offenders (
            id, full_name, cpf, birth_date, city_id,
            imprisoned, occupation,
            is_public_security_agent, security_force,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            education_level, observation,
            is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9::security_force_enum, $10, $11, $12, $13, $14::education_level_enum, $15, false
        )",
    )
    .bind(id)
    .bind(full_name)
    .bind(Option::<String>::None) // cpf
    .bind(Option::<NaiveDate>::None) // birth_date
    .bind(city_id)
    .bind(false) // imprisoned
    .bind(Option::<String>::None) // occupation
    .bind(false) // is_public_security_agent
    .bind(Option::<String>::None) // security_force
    .bind(false) // uses_alcohol
    .bind(false) // uses_drugs
    .bind(false) // has_psychiatric_issues
    .bind(Option::<Vec<String>>::None) // psychiatric_issues_type
    .bind("Elementary") // education_level
    .bind(Option::<String>::None) // observation
    .execute(pool)
    .await
    .expect("Failed to insert test offender");
    id
}

/// Insert a test protective measure and return its id.
pub async fn insert_protective_measure(
    pool: &PgPool,
    victim_id: Uuid,
    offender_id: Uuid,
    court_district_id: Uuid,
    status: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO protective_measures (
            id, process_number, sei_process_number, occurrence_report_number, issued_at,
            judicial_authority, court_district_id, distance_meters, status, violence_types,
            relationship_to_victim, assaults_children, was_drunk_during_assault, victim_id, offender_id,
            is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9::protective_measure_status_enum, $10::violence_type_enum[],
            $11::relationship_to_victim_enum, $12, $13, $14, $15,
            false
        )",
    )
    .bind(id)
    .bind("2025.000.000001-0")
    .bind(Option::<String>::None)
    .bind(Option::<String>::None)
    .bind(NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid date"))
    .bind("1st Criminal Court")
    .bind(court_district_id)
    .bind(Option::<i32>::None)
    .bind(status)
    .bind(vec!["Physical".to_string()])
    .bind("Spouse")
    .bind(false)
    .bind(false)
    .bind(victim_id)
    .bind(offender_id)
    .execute(pool)
    .await
    .expect("Failed to insert test protective measure");
    id
}

/// Insert a test user associated with the given city and return its id.
pub async fn insert_user(
    pool: &PgPool,
    registration: &str,
    email: &str,
    profile: &str,
    city_id: Option<Uuid>,
) -> Uuid {
    use nupevid_api::core::policy_defaults;
    use nupevid_api::core::value_objects::profiles::Profile;

    let id = Uuid::new_v4();

    // Hash the password "senha123"
    let password = "senha123";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password for test user")
        .to_string();

    // Generate default policies for the profile
    let profile_enum: Profile = profile.try_into().expect("Invalid profile string");
    let policies = policy_defaults::default_for_profile(&profile_enum, city_id);
    let policies_json = serde_json::to_value(&policies).expect("Failed to serialize policies");

    sqlx::query(
        "INSERT INTO users (
            id, rank, registration, full_name, profile, email, password, city_id, permission_policies, is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, false
        )",
    )
    .bind(id)
    .bind("SD PM")
    .bind(registration)
    .bind("Test User")
    .bind(profile)
    .bind(email)
    .bind(password_hash)
    .bind(city_id)
    .bind(policies_json)
    .execute(pool)
    .await
    .expect("Failed to insert test user");
    id
}
