use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_database(database_url: &str, max_connections: u32) -> PgPool {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await
        .expect("Failed to create pool database")
}
