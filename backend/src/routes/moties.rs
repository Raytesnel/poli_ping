use crate::app::AppState;
use crate::models;
use crate::services::motie_services;
use axum::{Json, Router, extract::Query, extract::State, http::StatusCode, routing::get};
use shared::MotieDto;
use shared::{GET_NEXT_MOTIE, GET_MOTIES};

const USER_ID: &str = "dev-user";

pub fn routes() -> Router<AppState>  {
    Router::new()
        .route(GET_MOTIES, get(get_moties))
        .route(GET_NEXT_MOTIE, get(get_nex_user_motie))
}

async fn get_moties(
    Query(motion_param): Query<models::api_models::GetMotiesQuery>,
) -> Result<Json<Vec<MotieDto>>, StatusCode> {
    let max_number = motion_param.max_number.unwrap_or(100);
    motie_services::get_moties(&max_number).await
}

async fn get_nex_user_motie(State(state): State<AppState>) -> Result<Json<MotieDto>, StatusCode> {
    let motie = motie_services::get_next_motie(&state.pool, USER_ID)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(motie))
}
