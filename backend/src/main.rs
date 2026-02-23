use sqlx::SqlitePool;
use crate::services::motie_services;

mod app;
mod models;
mod repository;
mod routes;
mod services;


pub async fn open_sqlite_pool() -> SqlitePool {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL_INTERNAL")
        .expect("DATABASE_URL must be set");

    println!("DATABASE_URL: {database_url}");

    let db_path = database_url.strip_prefix("sqlite://")
        .unwrap_or(&database_url);

    println!("Resolved DB path: {db_path}");
    let abs_path = std::env::current_dir().unwrap().join(db_path);
    println!("Absolute path: {:?}", abs_path);
    &std::env::var("DATABASE_URL_INTERNAL").unwrap();

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

    let pool = open_sqlite_pool()
        .await;
    motie_services::sync_latest_moties(&pool, 200)
        .await
        .expect("Failed to sync moties");

    app::run(pool).await;
}
