use axum::{http::StatusCode, Json};
use reqwest::Client;
use sqlx::SqlitePool;
use crate::repository::motie;

use crate::models::api_models::{ApiResponse, MotieTransformed};
use shared::{MotieDto, MotieProgressDto, VoteDto};
use crate::repository::motie::{existing_ids};
use crate::services::llm::convert_with_llm;



pub async fn get_moties(max_number: &u16) -> Result<Json<Vec<MotieTransformed>>, StatusCode> {

    let moties = fetch_moties_from_api(max_number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = transform_moties(moties)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn fetch_moties_from_api(max_number: &u16) -> Result<ApiResponse, anyhow::Error> {
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

async fn transform_moties(moties: ApiResponse) -> Result<Vec<MotieTransformed>, anyhow::Error> {
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
            let motie = MotieTransformed {
                external_id: m.id,
                title: m.onderwerp.unwrap_or_else(|| "Unknown".to_string()),
                description: m.titel, // Weird but true. they put more info in title and some title in description...
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

pub async fn sync_latest_moties(
    pool: &SqlitePool,
    max_number: u16,
) -> Result<(), anyhow::Error> {
    let api_response = fetch_moties_from_api(&max_number).await?;
    let transformed = transform_moties(api_response).await?;
    let ids: Vec<String> = transformed.iter()
        .map(|m| m.external_id.clone())
        .collect();

    let existing_id_list = existing_ids(pool, &ids).await?;
    let existing: std::collections::HashSet<_> = existing_id_list.into_iter().collect();

    let new_moties: Vec<_> = transformed
        .into_iter()
        .filter(|m| !existing.contains(&m.external_id))
        .collect();

    for mut motie in new_moties {
        let llm_response = convert_with_llm(&motie).await;
        motie = MotieTransformed{
            external_id:motie.external_id,
            title:llm_response.titel_kort,
            result: motie.result,
            description: llm_response.beschrijving,
            votes: motie.votes,
            timestamp: motie.timestamp.clone(),

        };
        let motie_id = motie::insert_motie(pool, &motie).await?;

        for vote in &motie.votes {
            motie::insert_party_vote(
                pool,
                motie_id,
                &vote.party,
                &vote.vote,
            )
                .await?;
        }
    }

    Ok(())
}


pub async fn get_next_motie(pool: &SqlitePool, user_id: &str) -> Result<MotieDto, anyhow::Error> {
    let motie = motie::get_next_unseen_motie(pool, &user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No more moties"))?;
    let votes = motie::get_party_votes(pool, motie.id).await?;

    Ok(MotieDto {
        id: motie.id as i32,
        title: motie.title,
        description: motie.description,
        result: motie.result,
        timestamp: motie.timestamp.to_string(),
        votes: votes
            .into_iter()
            .map(|v| VoteDto {
                party: v.party,
                vote: v.vote,
            })
            .collect(),
    })
}

pub async fn get_user_motie_progress(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<MotieProgressDto, sqlx::Error> {
    let voted: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM user_votes WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let total: (i32,) = sqlx::query_as("SELECT COUNT(*) FROM moties")
        .fetch_one(pool)
        .await?;

    Ok(MotieProgressDto {
        voted: voted.0,
        total: total.0,
    })
}