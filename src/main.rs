use std::collections::HashMap;
use reqwest::Client;
use axum::{
    routing::get,
    Router,
    Json
};
use serde_json::Value;

async fn root()->Json<HashMap<String, String>> {
    Json(HashMap::from([
    ("sentence".to_owned(), "hello World".to_owned()),
]))
}
async fn get_moties() {
    let url : &str = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Zaak\
        ?$filter=verwijderd%20eq%20false%20and%20Soort%20eq%20'Motie'\
        &$orderby=GewijzigdOp%20desc\
        &$top=200\
        &$expand=Besluit($expand=Stemming($expand=Fractie))";
    let client = Client::new();
    let response: Value = client
        .get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?
        .json()
        .await?;

    let mut moties = Vec::new();
    let moties: &Vec<Value> = response["value"].as_array().unwrap_or(&vec![]);
    for motie in moties {
        let besluiten = match motie.get("Besluit").and_then(|b| b.as_array()) {
            Some(b) if !b.is_empty() => b,
            _ => continue,
        };

        for besluit in besluiten {
            let stemming = besluit.get("Stemming").and_then(|s| s.as_array());
            if stemming.is_none() || stemming.unwrap().is_empty() {
                continue;
            }

            let votes: Vec<Value> = stemming.unwrap().iter().map(|s| {
                serde_json::json!({
                    "party": s["ActorFractie"],
                    "vote": s["Soort"]
                })
            }).collect();

            let motie = serde_json::json!({
                "id": motie["Id"],
                "title": motie["Titel"],
                "description": motie.get("Onderwerp"),
                "result": besluit["BesluitTekst"],
                "timestamp": motie["GewijzigdOp"],
                "votes": votes
            });

            moties.push(motie);
            break;
        }
    }

    Ok(moties)

}
async fn post_foo() {}
async fn foo_bar() {}


#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/foo", get(get_moties).post(post_foo))
        .route("/foo/bar", get(foo_bar));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
