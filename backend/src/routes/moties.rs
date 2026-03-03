use crate::app::AppState;
use crate::models;
use crate::services::motie_services;
use axum::{extract::Query, extract::State, http::StatusCode, routing::get, routing::post, Json, Router};
use shared::{MotieDto, MotieProgressDto, UserIdRequest, GET_MOTIE_PROGRESS};
use shared::{GET_MOTIES, GET_NEXT_MOTIE};
use crate::models::api_models::MotieTransformed;


pub fn routes() -> Router<AppState>  {
    Router::new()
        .route(GET_MOTIES, get(get_moties))
        .route(GET_NEXT_MOTIE, post(get_nex_user_motie))
        .route(GET_MOTIE_PROGRESS, post(get_user_progress))
}

async fn get_moties(
    Query(motion_param): Query<models::api_models::GetMotiesQuery>,
) -> Result<Json<Vec<MotieTransformed>>, StatusCode> {
    let max_number = motion_param.max_number.unwrap_or(100);
    motie_services::get_moties(&max_number).await
}


async fn get_nex_user_motie(State(state): State<AppState>, Json(req): Json<UserIdRequest>,) -> Result<Json<MotieDto>, StatusCode> {
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