use crate::models;
use crate::services::motie_services;
use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use shared::MotieDto;
use shared::{GET_MOTIES,GET_FIRST_MOTIE};
pub fn routes() -> Router {
    Router::new()
        .route(GET_MOTIES, get(get_moties))
        .route(GET_FIRST_MOTIE, get(get_first_motie))
}

async fn get_moties(
    Query(motion_param): Query<models::api_models::GetMotiesQuery>,
) -> Result<Json<Vec<MotieDto>>, StatusCode> {
    let max_number = motion_param.max_number.unwrap_or(100);
    motie_services::get_moties(&max_number).await
}

async fn get_first_motie(
) -> Result<Json<MotieDto>, StatusCode> {
    let max_number = 200;

    let moties = motie_services::get_moties(&max_number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(first) = moties.first().cloned() {
        Ok(Json(first))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
