use crate::app::AppState;
use crate::models::api_models::MotieTransformed;
use crate::services::motie_services;
use axum::{Json, Router, extract::State, http::StatusCode, routing::get, routing::post};
use shared::{GET_MOTIE_PROGRESS, MotieDto, MotieProgressDto, UserIdRequest};
use shared::{GET_MOTIES, GET_NEXT_MOTIE};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(GET_MOTIES, get(get_moties))
        .route(GET_NEXT_MOTIE, post(get_nex_user_motie))
        .route(GET_MOTIE_PROGRESS, post(get_user_progress))
}

async fn get_moties(
    State(state): State<AppState>,
) -> Result<Json<Vec<MotieTransformed>>, StatusCode> {
    motie_services::get_moties(state.api.as_ref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_nex_user_motie(
    State(state): State<AppState>,
    Json(req): Json<UserIdRequest>,
) -> Result<Json<MotieDto>, StatusCode> {
    let motie = motie_services::get_next_motie(&state.pool, &req.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(motie))
}

async fn get_user_progress(
    State(state): State<AppState>,
    Json(req): Json<UserIdRequest>,
) -> Result<Json<MotieProgressDto>, StatusCode> {
    let progress = motie_services::get_user_motie_progress(&state.pool, &req.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(progress))
}
