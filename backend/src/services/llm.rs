use reqwest::Client;
use serde_json::json;

use serde::{Deserialize, Serialize};
use crate::models::api_models::MotieTransformed;

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub titel_kort: String,
    pub kamerleden: Vec<String>,
    pub beschrijving: String,
    pub thema: String,
    pub tags: Vec<String>,
}

async fn call_gemini(prompt: &str) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let api_key = std::env::var("LLM_KEY").expect("LLM_KEY must be set");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
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


pub async fn convert_with_llm(motie:&MotieTransformed) ->LlmResponse {
    println!("lets convert motie: {motie:?}");
    let minimal = json!({
  "title": motie.title,
  "description": motie.description
});
    let prompt = format!(r#"
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
{}

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
- beschrijving: neutrale samenvatting in één of twee zinnen
- tags: relevante onderwerpen (3-6 tags)
- thema: breed beleidsdomein (bijv. Economie, Migratie, Binnenlands bestuur)
- als informatie ontbreekt: laat veld leeg of gebruik null
- Je output moet altijd valide JSON zijn zonder codeblock.
- Gebruik dubbele quotes.
- Geen trailing komma's.
- Als informatie ontbreekt: gebruik null.
- geen Markdown of extra tekst
"#,minimal);
    // how to add the

    match call_gemini(&prompt).await {
        Ok(resp) => {
            extract_text(&resp).unwrap()
        }
        Err(e) => panic!("Error: {}", e),
    }
}