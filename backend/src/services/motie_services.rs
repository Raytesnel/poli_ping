use axum::{Json, http::StatusCode};
use reqwest::Client;

use crate::models::api_models::ApiResponse;
use shared::{MotieDto, VoteDto};

pub async fn get_moties(max_number: &u16) -> Result<Json<Vec<MotieDto>>, StatusCode> {
    let moties = fetch_moties_from_api(&max_number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let result = transform_moties(moties).await?;
    Ok(Json(result))
}

async fn fetch_moties_from_api(max_number: &u16) -> Result<ApiResponse, reqwest::Error> {
    let url = format!(
        "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Zaak?$filter=verwijderd%20eq%20false%20and%20Soort%20eq%20'Motie'&$orderby=GewijzigdOp%20desc&$top={}&$expand=Besluit($expand=Stemming($expand=Fractie))",
        max_number
    );
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

async fn transform_moties(moties: ApiResponse) -> Result<Vec<MotieDto>, StatusCode> {
    let mut result = Vec::new();
    for m in moties.value {
        if m.besluit.is_empty() {
            continue;
        }
        for besluit in m.besluit {
            if besluit.stemming.is_empty() {
                continue;
            }
            let votes: Vec<VoteDto> = besluit
                .stemming
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
            break;
        }
    }

    Ok(result)
}
