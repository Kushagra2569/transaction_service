use dotenv::dotenv;
use sqlx::PgPool;
use sqlx::{postgres::PgPoolOptions, Pool};
use std::env;
use tokio::sync::OnceCell;
use tracing::{error, info, instrument};

static CONN: OnceCell<PgPool> = OnceCell::const_new();

pub async fn connect_db() -> Result<Pool<sqlx::Postgres>, sqlx::Error> {
    dotenv().ok();
    let db_url = env::var("POSTGRES_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;
    Ok(pool)
}

#[instrument]
async fn init_conn() -> PgPool {
    let db_pool = connect_db();
    let db_conn_info;
    match db_pool.await {
        Ok(value) => {
            info!("Db Connection Successful");
            db_conn_info = value;
        }
        Err(err) => {
            error!("error {}", err);
            panic!("Database connection was not established");
        }
    }
    db_conn_info
}
pub async fn get_conn() -> &'static PgPool {
    CONN.get_or_init(init_conn).await
}
