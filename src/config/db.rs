use dotenv::dotenv;
use sqlx::{postgres::PgPoolOptions, Pool, Row};
use std::env;

pub async fn connect_db() -> Result<Pool<sqlx::Postgres>, sqlx::Error> {
    dotenv().ok();
    let db_url = env::var("POSTGRES_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;

    Ok(pool)
}
