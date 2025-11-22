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
    .bind("SP")
    .bind("1º BPM")
    .execute(pool)
    .await
    .expect("Failed to insert test city");
    id
}

/// Insert a test victim associated with the given city and return its id.
pub async fn insert_victim(pool: &PgPool, full_name: &str, city_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO victims (id, full_name, document_id, birth_date, phone, city_id, is_deleted) \
         VALUES ($1, $2, $3, $4, $5, $6, false)",
    )
    .bind(id)
    .bind(full_name)
    .bind(Option::<String>::None)
    .bind(Option::<NaiveDate>::None)
    .bind(Option::<String>::None)
    .bind(city_id)
    .execute(pool)
    .await
    .expect("Failed to insert test victim");
    id
}
