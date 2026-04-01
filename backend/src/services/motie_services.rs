use crate::models::api_models::{ApiResponse, MotieTransformed};
use crate::repository::motie;
use crate::repository::motie::existing_ids;
use crate::services::llm::LlmService;
use async_trait::async_trait;
use axum::{Json, http::StatusCode};
use chrono::Local;
use reqwest::Client;
use shared::{MotieDocumentDto, MotieDto, MotieProgressDto, VoteDto};
use sqlx::SqlitePool;
use tracing::{debug, info};

#[async_trait]
pub trait MotieApi: Send + Sync {
    async fn fetch_moties(&self) -> Result<ApiResponse, anyhow::Error>;
}
pub struct RealMotieApi;
#[async_trait]
impl MotieApi for RealMotieApi {
    async fn fetch_moties(&self) -> Result<ApiResponse, anyhow::Error> {
        fetch_moties_from_api().await
    }
}

pub async fn get_moties(api: &dyn MotieApi) -> Result<Json<Vec<MotieTransformed>>, StatusCode> {
    let moties = api
        .fetch_moties()
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
        date = date
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
        json
    })
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

            let documents: Vec<MotieDocumentDto> = m
                .document
                .iter()
                .map(|d| MotieDocumentDto {
                    document_id: d.id.clone(),
                })
                .collect();

            let motie = MotieTransformed {
                external_id: m.id,
                title: m.onderwerp.unwrap_or_else(|| "Unknown".to_string()),
                description: m.titel, // Weird but true. they put more info in title and some title in description...
                result: besluit_result.trim_end_matches('.').to_string(),
                timestamp: m.gewijzigd_op,
                votes,
                documents,
            };

            result.push(motie);
            break;
        }
    }

    Ok(result)
}

pub async fn sync_latest_moties(
    pool: &SqlitePool,
    api: &dyn MotieApi,
    llm: &dyn LlmService,
) -> Result<(), anyhow::Error> {
    let api_response = api.fetch_moties().await?;
    let transformed = transform_moties(api_response).await?;
    let ids: Vec<String> = transformed.iter().map(|m| m.external_id.clone()).collect();

    let existing_id_list = existing_ids(pool, &ids).await?;
    let existing: std::collections::HashSet<_> = existing_id_list.into_iter().collect();

    let new_moties: Vec<_> = transformed
        .into_iter()
        .filter(|m| !existing.contains(&m.external_id))
        .collect();

    for mut motie in new_moties {
        let llm_response = llm.convert(&motie).await;
        motie = MotieTransformed {
            external_id: motie.external_id,
            title: llm_response.titel_kort,
            result: motie.result,
            description: llm_response.beschrijving,
            votes: motie.votes,
            timestamp: motie.timestamp.clone(),
            documents: motie.documents,
        };
        let motie_id = motie::insert_motie(pool, &motie).await?;

        for vote in &motie.votes {
            motie::insert_party_vote(pool, motie_id, &vote.party, &vote.vote).await?;
        }
        for document in &motie.documents {
            motie::insert_documents(&document.document_id, motie_id, pool).await?;
        }
    }

    Ok(())
}

pub async fn get_next_motie(pool: &SqlitePool, user_id: &str) -> Result<MotieDto, anyhow::Error> {
    let motie = motie::get_next_unseen_motie(pool, user_id)
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
        document_id: document.iter().map(|f| f.document_id.clone()).collect(),
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::api_models::{Besluit, Document, Stemming, Zaak};
    use crate::services::llm::LlmResponse;
    use std::sync::{Arc, Mutex};

    struct MockApi {
        response: ApiResponse,
    }

    #[async_trait]
    impl MotieApi for MockApi {
        async fn fetch_moties(&self) -> Result<ApiResponse, anyhow::Error> {
            Ok(self.response.clone())
        }
    }

    struct MockLlm {
        response: LlmResponse,
        calls: Arc<Mutex<u32>>,
    }
    #[async_trait]
    impl LlmService for MockLlm {
        async fn convert(&self, _motie: &MotieTransformed) -> LlmResponse {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;
            self.response.clone()
        }
    }

    fn example_api_response_with_one_motie() -> ApiResponse {
        ApiResponse {
            value: vec![Zaak {
                id: "123".to_string(),
                titel: "".to_string(),
                onderwerp: Some("test".to_string()),
                gewijzigd_op: "2024-6-10".to_string(),
                besluit: vec![Besluit {
                    id: "1234".to_string(),
                    besluit_tekst: Some("Aangenomen.".to_string()),
                    stemming: vec![
                        Stemming {
                            id: "1234".to_string(),
                            soort: "voor".to_string(),
                            actor_fractie: Some("D66".to_string()),
                        },
                        Stemming {
                            id: "1234".to_string(),
                            soort: "tegen".to_string(),
                            actor_fractie: Some("PVV".to_string()),
                        },
                    ],
                }],
                document: vec![Document {
                    id: "123".to_string(),
                }],
            }],
        }
    }
    fn example_llm_response() -> LlmResponse {
        LlmResponse {
            titel_kort: "SHORT TITLE".to_string(),
            beschrijving: "mock description".to_string(),
            kamerleden: vec!["D66".to_string(), "ProNL".to_string()],
            thema: "mock description".to_string(),
            tags: vec!["mock".to_string(), "test".to_string()],
        }
    }
    fn mock_api_with(data: ApiResponse) -> MockApi {
        MockApi { response: data }
    }

    fn mock_llm() -> MockLlm {
        MockLlm {
            response: example_llm_response(),
            calls: Arc::new(Mutex::new(0)),
        }
    }
    #[sqlx::test]
    async fn sync_latest_moties_with_moties_returns_list_moties(pool: SqlitePool) {
        let api = mock_api_with(example_api_response_with_one_motie());
        let llm = mock_llm();

        sync_latest_moties(&pool, &api, &llm).await.unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM moties")
            .fetch_one(&pool)
            .await
            .unwrap();
        println!("count: {:?}", count);
        assert!(count.0 > 0);
    }
    fn empty_api_response() -> ApiResponse {
        ApiResponse { value: vec![] }
    }
    #[sqlx::test]
    async fn sync_latest_moties_witouth_moties_returns_empty_list(pool: SqlitePool) {
        let api = mock_api_with(empty_api_response());
        let llm = mock_llm();

        sync_latest_moties(&pool, &api, &llm).await.unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM moties")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 0);
    }

    #[sqlx::test]
    async fn sync_latest_moties_no_new_moties(pool: SqlitePool) {
        let api = mock_api_with(example_api_response_with_one_motie());
        let llm = mock_llm();

        // first insert
        sync_latest_moties(&pool, &api, &llm).await.unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM moties")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 1);
        // second run (should not insert duplicates)
        sync_latest_moties(&pool, &api, &llm).await.unwrap();

        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM moties")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(count.0, 1);
    }
    #[sqlx::test]
    async fn sync_calls_llm(pool: SqlitePool) {
        let api = mock_api_with(example_api_response_with_one_motie());

        let calls = Arc::new(Mutex::new(0));
        let llm = MockLlm {
            response: example_llm_response(),
            calls: calls.clone(),
        };

        sync_latest_moties(&pool, &api, &llm).await.unwrap();

        assert_eq!(*calls.lock().unwrap(), 1);
    }
}
