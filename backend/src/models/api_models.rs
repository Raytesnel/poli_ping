use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResponse {
    pub value: Vec<Zaak>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Zaak {
    pub id: String,
    pub titel: String,
    pub onderwerp: Option<String>,
    pub gewijzigd_op: String,
    pub besluit: Vec<Besluit>,
    pub document: Vec<Document>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Document {
    pub id: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Besluit {
    pub id: String,
    #[serde(rename = "Agendapunt_Id")]
    pub besluit_tekst: Option<String>,
    pub stemming: Vec<Stemming>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Stemming {
    pub id: String,
    pub soort: String,
    pub actor_fractie: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MotieTransformed {
    pub external_id: String,
    pub title: String,
    pub description: String,
    pub result: String,
    pub timestamp: String,
    pub votes: Vec<shared::VoteDto>,
    pub documents: Vec<shared::MotieDocumentDto>,
}
