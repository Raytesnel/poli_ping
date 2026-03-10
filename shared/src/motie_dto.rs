use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct MotieDto {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub result: String,
    pub timestamp: String,
    pub votes: Vec<VoteDto>,
    pub document_id: Vec<String>,
}

#[derive(Debug, Serialize,Deserialize,Clone,PartialEq)]
pub struct VoteDto {
    pub party: String,
    pub vote: String,
}

#[derive(Debug,Serialize, Deserialize, Clone)]
pub struct MotieDocumentDto {
    pub document_id: String,
}
#[derive(Debug,Serialize,Deserialize)]
pub struct UserIdRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize,Deserialize, Clone, PartialEq)]
pub struct MotieProgressDto {
    pub voted: i32,
    pub total: i32,
}