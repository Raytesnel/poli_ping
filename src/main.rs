use axum::{
    http::StatusCode,
    routing::get,
    Json,
    Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    value: Vec<Zaak>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Zaak {
    id: String,
    nummer: Option<String>,
    soort: String,
    titel: String,
    citeertitel: Option<String>,
    alias: Option<String>,
    status: String,
    onderwerp: Option<String>,
    gestart_op: Option<String>,
    organisatie: String,
    grondslagvoorhang: Option<String>,
    termijn: Option<String>,
    vergaderjaar: Option<String>,
    volgnummer: Option<i64>,
    huidige_behandelstatus: Option<String>,
    afgedaan: Option<bool>,
    groot_project: Option<bool>,
    gewijzigd_op: String,
    api_gewijzigd_op: String,
    verwijderd: bool,
    kabinetsappreciatie: Option<String>,
    besluit: Vec<Besluit>,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Besluit {
    id: String,
    #[serde(rename = "Agendapunt_Id")]
    agendapunt_id: String,
    stemmings_soort: Option<String>,
    besluit_soort: Option<String>,
    besluit_tekst: Option<String>,
    opmerking: Option<String>,
    status: Option<String>,
    agendapunt_zaak_besluit_volgorde: Option<i64>,
    gewijzigd_op: String,
    api_gewijzigd_op: String,
    verwijderd: bool,
    stemming: Vec<Stemming>,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Stemming {
    id: String,
    besluit_id: Option<String>,
    soort: String,
    status: Option<String>,
    actor_naam: Option<String>,
    actor_fractie: Option<String>,

}

async fn fetch_moties_from_api() -> Result<ApiResponse, reqwest::Error> {
    let url = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Zaak?$filter=verwijderd%20eq%20false%20and%20Soort%20eq%20'Motie'&$orderby=GewijzigdOp%20desc&$top=199&$expand=Besluit($expand=Stemming($expand=Fractie))";

    let client = Client::new();

    let json: ApiResponse = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    Ok(json)
}
#[derive(Debug, Serialize)]
struct MotieDto {
    id: String,
    title: String,
    description: Option<String>,
    result: String,
    timestamp: String,
    votes: Vec<VoteDto>,
}

#[derive(Debug, Serialize)]
struct VoteDto {
    party: String,
    vote: String,
}

async fn root()->Json<HashMap<String, String>> {
    Json(HashMap::from([
    ("sentence".to_owned(), "hello World".to_owned()),
]))
}

async fn get_moties() -> Result<Json<Vec<MotieDto>>, StatusCode> {
    let moties = fetch_moties_from_api()
        .await
        .map_err(|e| {
            eprintln!("Error fetching moties: {e}");
            StatusCode::BAD_GATEWAY
        })?;

    let mut result = Vec::new();

    for m in moties.value {
        if m.besluit.is_empty() {
            continue;
        }
        for besluit in m.besluit {
            if besluit.stemming.is_empty() {
                continue;
            }
            let votes: Vec<VoteDto> = besluit.stemming
                .into_iter()
                .map(|s| VoteDto {
                    party: s.actor_fractie.unwrap_or_else(|| "Unknown".to_string()),
                    vote: s.soort,
                })
                .collect();
            let Some(besluit_result) = besluit
                .besluit_tekst
                .filter(|r| matches!(r.as_str(), "Aangenomen." | "Verworpen."))
            else {
                continue;
            };
            let motie = MotieDto {
                id: m.id,
                title: m.titel,
                description: m.onderwerp,
                result: besluit_result.trim_end_matches('.').to_string(),
                timestamp: m.gewijzigd_op,
                votes,
            };

            result.push(motie);
            break; // only first besluit with votes
        }
    }

    Ok(Json(result))
}


#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/get_moties", get(get_moties));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
