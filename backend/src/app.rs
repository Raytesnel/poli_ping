use axum::Router;

use crate::routes::moties;

pub async fn run() {
    let app = Router::new()
        .merge(moties::routes());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}