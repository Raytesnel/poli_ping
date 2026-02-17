use axum::{routing::get, Router,extract::Query};

use crate::services::motie_services;
use crate::models;
use shared::MotieDto;
use shared::GET_MOTIES;
pub fn routes() -> Router {
    Router::new()
        .route(GET_MOTIES, get(get_moties))
}

async fn get_moties(Query(motion_param): Query<models::api_models::GetMotiesQuery>) -> Result<axum::Json<Vec<MotieDto>>, axum::http::StatusCode> {
    let max_number = motion_param.max_number.unwrap_or(100);
    motie_services::get_moties(&max_number).await
}