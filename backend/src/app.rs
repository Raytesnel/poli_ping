use crate::routes::moties;
use crate::routes::votes;
use axum::Router;
use sqlx::SqlitePool;

use tower_http::cors::{Any, CorsLayer};
use shared::BASE_URL_BACKEND;

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

    let listener = tokio::net::TcpListener::bind(BASE_URL_BACKEND).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
