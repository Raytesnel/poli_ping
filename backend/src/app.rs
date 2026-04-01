use crate::routes::moties;
use crate::routes::votes;
use axum::Router;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::services::llm::{LlmService, RealLlmService};
use crate::services::motie_services::{MotieApi, RealMotieApi};
use shared::BASE_URL_BACKEND;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub api: Arc<dyn MotieApi + Send + Sync>,
    pub llm: Arc<dyn LlmService + Send + Sync>,
}

pub fn create_app(pool: SqlitePool) -> (Router, AppState) {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState {
        pool,
        api: Arc::new(RealMotieApi),
        llm: Arc::new(RealLlmService),
    };

    let app = Router::new()
        .merge(moties::routes())
        .merge(votes::routes())
        .layer(cors)
        .with_state(state.clone());
    (app, state)
}

pub async fn run(app: Router) {
    let listener = tokio::net::TcpListener::bind(BASE_URL_BACKEND)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
