use crate::routes::moties;
use crate::routes::votes;
use axum::Router;
use sqlx::SqlitePool;

use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

pub async fn run(pool: SqlitePool) {
    let cors = CorsLayer::new()
        .allow_origin(Any) // allow all origins for dev
        .allow_methods(Any)
        .allow_headers(Any);
    let state = AppState { pool };
    let app = Router::new()
        .merge(moties::routes())
        .merge(votes::routes())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
