use crate::app::AppState;
use crate::repository::vote::insert_user_vote;
use axum::{http::StatusCode, routing::post, Json, Router};
use axum::extract::State;
use shared::POST_USER_VOTE;
use shared::AddUserVoteRequest;

pub fn routes() -> Router<AppState> {
    Router::new().route(POST_USER_VOTE, post(add_user_vote))
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
