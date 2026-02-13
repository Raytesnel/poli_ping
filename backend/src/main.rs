mod app;
mod routes;
mod services;
mod models;

#[tokio::main]
async fn main() {
    app::run().await;
}