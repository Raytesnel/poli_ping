use crate::services::motie_services;
use sqlx::SqlitePool;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;

mod app;
mod models;
mod repository;
mod routes;
mod services;


pub async fn open_sqlite_pool() -> SqlitePool {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    info!("DATABASE_URL: {database_url}");

    let db_path = database_url.strip_prefix("sqlite://")
        .unwrap_or(&database_url);

    debug!("Resolved DB path: {db_path}");
    let abs_path = std::env::current_dir().unwrap().join(db_path);
    debug!("Absolute path: {:?}", abs_path);

    SqlitePool::connect(&database_url).await.unwrap_or_else(|err| {
        panic!(
            "Failed to open SQLite database.\n\
             DATABASE_URL: {}\n\
             Resolved path: {}\n\
             Absolute path: {:?}\n\
             Error: {}",
            database_url, db_path, abs_path, err
        )
    })
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let log_level = std::env::var("RUST_LOG").expect("RUST_LOG must be set");
    tracing_subscriber::fmt()
        .with_env_filter(log_level.clone())
        .compact()
        .init();
    debug!("log level: {log_level}");
    let pool = open_sqlite_pool()
        .await;
    motie_services::sync_latest_moties(&pool)
        .await
        .expect("Failed to sync moties");

    app::run(pool).await;
}
