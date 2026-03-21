use serde::{Deserialize, Serialize};
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
pub struct Motie {
    pub id: i64,
    #[allow(dead_code)]
    pub external_id: String,
    pub title: String,
    pub description: String,
    pub result: String,
    pub timestamp: String, // store as ISO8601 string in SQLite
}
#[allow(dead_code)]
#[derive(sqlx::FromRow)]
pub struct PartyVote {
    pub id: i64,
    pub motie_id: i64,
    pub party: String,
    pub vote: String,
}
#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Serialize, Deserialize, Clone)]
pub struct MotieDocument {
    pub id: i64,
    pub motie_id: i64,
    pub document_id: String,
}
#[allow(dead_code)]
#[derive(sqlx::FromRow, Deserialize, Serialize)]
pub struct UserVotes {
    pub user_id: String,
    pub motie_id: i64,
    pub vote: String,
}
