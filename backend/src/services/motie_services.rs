use crate::repository::motie;
use axum::{http::StatusCode, Json};
use chrono::Local;
use reqwest::Client;
use sqlx::SqlitePool;
use crate::models::api_models::{ApiResponse, MotieTransformed};
use crate::repository::motie::existing_ids;
use crate::services::llm::convert_with_llm;
use shared::{MotieDocumentDto, MotieDto, MotieProgressDto, VoteDto};
use crate::models::db_models::MotieDocument;
use tracing::{debug, error, info, warn};

pub async fn get_moties() -> Result<Json<Vec<MotieTransformed>>, StatusCode> {

    let moties = fetch_moties_from_api()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let result = transform_moties(moties)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

async fn fetch_moties_from_api() -> Result<ApiResponse, anyhow::Error> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let url = format!(
        "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Zaak?$filter=Verwijderd%20eq%20false%20and%20Soort%20eq%20%27Motie%27%20and%20ApiGewijzigdOp%20ge%20{date}%20and%20Besluit/any(b:%20b/Stemming/any())&$orderby=GewijzigdOp%20desc&$expand=Besluit($expand=Stemming($expand=Fractie)),Document",
        date=date
    );
    info!("Fetching moties");
    debug!("Fetching moties from api: {}", url);
    let client = Client::new();

    let json: ApiResponse = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    Ok({
        info!("Done fetching moties from api");
        json })
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

            let documents: Vec<MotieDocumentDto> = m.document.iter().map(|d| MotieDocumentDto{
                document_id:d.id.clone(),
            })
            .collect();

            let motie = MotieTransformed {
                external_id: m.id,
                title: m.onderwerp.unwrap_or_else(|| "Unknown".to_string()),
                description: m.titel, // Weird but true. they put more info in title and some title in description...
                result: besluit_result.trim_end_matches('.').to_string(),
                timestamp: m.gewijzigd_op,
                votes,
                documents:documents
            };

            result.push(motie);
            break;
        }
    }

    Ok(result)
}

pub async fn sync_latest_moties(
    pool: &SqlitePool,
) -> Result<(), anyhow::Error> {
    let api_response = fetch_moties_from_api().await?;
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
            documents: motie.documents

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
        for document in &motie.documents {
            motie::insert_documents(
                &document.document_id,
                motie_id,
                pool,

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
    let document = motie::get_document_ids(pool, &motie.id).await?;
    Ok(MotieDto {
        id: motie.id as i32,
        title: motie.title,
        description: motie.description,
        result: motie.result,
        timestamp: motie.timestamp.to_string(),
        document_id:document.iter().map(|f| f.document_id.clone()).collect(),
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