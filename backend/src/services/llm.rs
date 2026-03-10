use pdf_extract::extract_text_from_mem;
use reqwest::Client;
use serde_json::json;
use tokio::time::{Duration, sleep};

use crate::models::api_models::MotieTransformed;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub titel_kort: String,
    pub kamerleden: Vec<String>,
    pub beschrijving: String,
    pub thema: String,
    pub tags: Vec<String>,
}

pub async fn call_gemini_with_retry(prompt: &str) -> Result<String, anyhow::Error> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match call_gemini(prompt).await {
            Ok(resp) => return Ok(resp),

            Err(e) => {
                let msg = e.to_string();
                if msg.contains("429") || msg.contains("quota") {
                    if attempts > 5 {
                        println!("Waiting is not helping.");
                        return Err(e.into());
                    }
                    println!("Rate limit hit, waiting 40 seconds...");
                    sleep(Duration::from_secs(40)).await;
                } else {
                    return Err(e.into());
                }
            }
        }
    }
}

async fn call_gemini(prompt: &str) -> Result<String, reqwest::Error> {
    let llm_model = std::env::var("LLM_MODEL").expect("LLM_MODEL must be set");
    let client = Client::new();
    let api_key = std::env::var("LLM_KEY").expect("LLM_KEY must be set");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}",
        api_key = api_key,
        model = llm_model
    );

    let body = json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    let resp = client.post(url).json(&body).send().await?;
    println!("status: {}", resp.status());

    let text = resp.text().await?;
    println!("body: {}", text);
    Ok(text)
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Deserialize)]
struct Part {
    text: String,
}

fn extract_text(response: &str) -> Option<LlmResponse> {
    let parsed: GeminiResponse = serde_json::from_str(response).ok()?;
    let text = parsed
        .candidates
        .first()?
        .content
        .parts
        .first()
        .map(|p| p.text.clone());

    serde_json::from_str(&text.unwrap()).ok()
}

pub async fn convert_with_llm(motie: &MotieTransformed) -> LlmResponse {
    println!("lets convert motie: {motie:?}");

    let document_text = get_document_text(&motie.documents)
        .await
        .unwrap_or_default();

    let minimal = json!({
      "title": motie.title,
      "description": motie.description
    });
    let prompt = format!(
        r#"
Je bent een parser die Kamerstukken omzet naar gestructureerde JSON.

Taak:
- Analyseer de motie / tekst
- Extraheer de belangrijkste informatie
- Geef alleen JSON terug
- Geen extra uitleg
Focus op:
- doel van de motie
- indieners
- beleidsgebied
- gewenste actie

Negeer administratieve metadata zoals t.v.v.-nummers.

Motie data:
{motie_data}
Motie information:
{document_text}
JSON structuur:
{{
  "titel_kort": "...",
  "kamerleden": ["..."],
  "beschrijving": "...",
  "tags": ["...", "..."],
  "thema": "..."
}}

Regels:
- titel_kort: korte, begrijpelijke titel (max ~80 tekens)
- kamerleden: lijst van indieners (achternamen)
- beschrijving: neutrale samenvatting in drie of vijf zinnen
- tags: relevante onderwerpen (3-6 tags)
- thema: breed beleidsdomein (bijv. Economie, Migratie, Binnenlands bestuur)
- als informatie ontbreekt: laat veld leeg of gebruik null
- Je output moet altijd valide JSON zijn zonder codeblock.
- Gebruik dubbele quotes.
- Geen trailing komma's.
- Als informatie ontbreekt: gebruik null.
- geen Markdown of extra tekst
"#,
        motie_data = minimal,
        document_text = document_text
    );
    // how to add the

    match call_gemini_with_retry(&prompt).await {
        Ok(resp) => extract_text(&resp).unwrap(),
        Err(e) => panic!("Error: {}", e),
    }
}

pub async fn download_document(document_id: &str) -> Result<Vec<u8>, anyhow::Error> {
    let url = format!(
        "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Document({})/resource",
        document_id
    );

    let bytes = reqwest::get(url).await?.bytes().await?;

    Ok(bytes.to_vec())
}

pub fn parse_pdf_text(pdf_bytes: &[u8]) -> Result<String, anyhow::Error> {
    let text = extract_text_from_mem(pdf_bytes)?;
    Ok(text)
}

pub async fn get_document_text(
    document_ids: &Vec<shared::MotieDocumentDto>,
) -> Result<String, anyhow::Error> {
    let mut combined_text = String::new();
    for document in document_ids {
        let pdf = download_document(&document.document_id).await?;
        let text = parse_pdf_text(&pdf)?;
        combined_text.push_str(&text);
        combined_text.push_str("\n\n"); // scheiding tussen documenten
    }
    println!("document extracted: \n\n{}", combined_text);
    Ok(combined_text)
}
