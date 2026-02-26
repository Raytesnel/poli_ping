use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize)]
pub struct AddUserVoteRequest {
    pub user_id: String,
    pub motie_id: i32,
    pub vote: String,
}