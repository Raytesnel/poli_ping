use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use crate::routes::moties;

pub async fn run() {

    let cors = CorsLayer::new()
        .allow_origin(Any) // allow all origins for dev
        .allow_methods(Any)
        .allow_headers(Any);


    let app = Router::new()
        .merge(moties::routes())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
