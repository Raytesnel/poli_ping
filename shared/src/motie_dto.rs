use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct MotieDto {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub result: String,
    pub timestamp: String,
    pub votes: Vec<VoteDto>,
}

#[derive(Debug, Serialize,Deserialize,Clone,PartialEq)]
pub struct VoteDto {
    pub party: String,
    pub vote: String,
}