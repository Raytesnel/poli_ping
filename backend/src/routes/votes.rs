use crate::app::AppState;
use crate::repository::vote::insert_user_vote;
use axum::{Json, Router, http::StatusCode, routing::post};
use axum::extract::State;
use serde::Deserialize;
use shared::POST_USER_VOTE;

pub fn routes() -> Router<AppState> {
    Router::new().route(POST_USER_VOTE, post(add_user_vote))
}

#[derive(Deserialize)]
pub struct AddUserVoteRequest {
    pub user_id: String,
    pub motie_id: i32,
    pub vote: String,
}

async fn add_user_vote(
    State(state): State<AppState>,
    Json(payload): Json<AddUserVoteRequest>,
) -> Result<Json<String>, StatusCode> {
    insert_user_vote(&state.pool, &payload.user_id, payload.motie_id, &payload.vote)
        .await
        .unwrap();
    Ok(Json("status: saved".to_string()))
}
