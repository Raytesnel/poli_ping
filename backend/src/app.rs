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

pub fn create_app(pool: SqlitePool) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { pool };

    Router::new()
        .merge(moties::routes())
        .merge(votes::routes())
        .layer(cors)
        .with_state(state)
}

pub async fn run(pool: SqlitePool) {
    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind(BASE_URL_BACKEND).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
