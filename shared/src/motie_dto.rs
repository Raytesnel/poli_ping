use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotieDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub result: String,
    pub timestamp: String,
    pub votes: Vec<VoteDto>,
}

#[derive(Debug, Serialize,Deserialize,Clone)]
pub struct VoteDto {
    pub party: String,
    pub vote: String,
}