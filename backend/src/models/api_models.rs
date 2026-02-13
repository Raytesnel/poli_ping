use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize)]
pub struct GetMotiesQuery {
    pub max_number: Option<u16>, // optional, default if not provided
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiResponse {
    pub value: Vec<Zaak>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Zaak {
    pub id: String,
    nummer: Option<String>,
    soort: String,
    pub titel: String,
    citeertitel: Option<String>,
    alias: Option<String>,
    status: String,
    pub onderwerp: Option<String>,
    gestart_op: Option<String>,
    organisatie: String,
    grondslagvoorhang: Option<String>,
    termijn: Option<String>,
    vergaderjaar: Option<String>,
    volgnummer: Option<i64>,
    huidige_behandelstatus: Option<String>,
    afgedaan: Option<bool>,
    groot_project: Option<bool>,
    pub gewijzigd_op: String,
    api_gewijzigd_op: String,
    verwijderd: bool,
    kabinetsappreciatie: Option<String>,
    pub besluit: Vec<Besluit>,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Besluit {
    id: String,
    #[serde(rename = "Agendapunt_Id")]
    agendapunt_id: String,
    stemmings_soort: Option<String>,
    besluit_soort: Option<String>,
    pub besluit_tekst: Option<String>,
    opmerking: Option<String>,
    status: Option<String>,
    agendapunt_zaak_besluit_volgorde: Option<i64>,
    gewijzigd_op: String,
    api_gewijzigd_op: String,
    verwijderd: bool,
    pub stemming: Vec<Stemming>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Stemming {
    id: String,
    besluit_id: Option<String>,
    pub soort: String,
    status: Option<String>,
    actor_naam: Option<String>,
    pub actor_fractie: Option<String>,

}