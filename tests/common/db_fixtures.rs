use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;

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
            education_level, occupation, workplace,
            violence_type, has_children, children_count,
            has_special_needs, special_needs_type,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9::violence_type_enum, $10::has_children_enum, $11, $12, $13, $14, $15, $16, $17, false
        )",
    )
    .bind(id)
    .bind(full_name)
    .bind(Option::<String>::None) // cpf
    .bind(Option::<NaiveDate>::None) // birth_date
    .bind(city_id)
    .bind(Option::<String>::None) // education_level
    .bind(Option::<String>::None) // occupation
    .bind(Option::<String>::None) // workplace
    .bind("Physical") // violence_type
    .bind("No") // has_children
    .bind(Option::<i32>::None) // children_count
    .bind(false) // has_special_needs
    .bind(Option::<String>::None) // special_needs_type
    .bind(false) // uses_alcohol
    .bind(false) // uses_drugs
    .bind(false) // has_psychiatric_issues
    .bind(Option::<String>::None) // psychiatric_issues_type
    .execute(pool)
    .await
    .expect("Failed to insert test victim");
    id
}

/// Insert a test offender associated with the given city and victim, return its id.
pub async fn insert_offender(pool: &PgPool, full_name: &str, city_id: Uuid, victim_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO offenders (
            id, full_name, cpf, birth_date, city_id, victim_id,
            imprisoned, occupation, workplace,
            is_public_security_agent, relationship_to_victim,
            uses_alcohol, uses_drugs,
            has_psychiatric_issues, psychiatric_issues_type,
            was_drunk_during_assault, observation,
            is_deleted
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::relationship_to_victim_enum, $12, $13, $14, $15, $16, $17, false
        )",
    )
    .bind(id)
    .bind(full_name)
    .bind(Option::<String>::None) // cpf
    .bind(Option::<NaiveDate>::None) // birth_date
    .bind(city_id)
    .bind(victim_id)
    .bind(false) // imprisoned
    .bind(Option::<String>::None) // occupation
    .bind(Option::<String>::None) // workplace
    .bind(false) // is_public_security_agent
    .bind("Spouse") // relationship_to_victim
    .bind(false) // uses_alcohol
    .bind(false) // uses_drugs
    .bind(false) // has_psychiatric_issues
    .bind(Option::<String>::None) // psychiatric_issues_type
    .bind(false) // was_drunk_during_assault
    .bind(Option::<String>::None) // observation
    .execute(pool)
    .await
    .expect("Failed to insert test offender");
    id
}
