use axum_test::TestServer;
use backend::app::create_app;
use sqlx::SqlitePool;
use uuid::Uuid;

pub fn test_server(pool: SqlitePool) -> TestServer {
    let (app, _) = create_app(pool);
    TestServer::new(app)
}

pub async fn seed_motie(pool: &SqlitePool) -> i64 {
    let random_id = Uuid::new_v4().to_string();
    sqlx::query!(
        r#"
        INSERT INTO moties (external_id,title,description,result,timestamp)
        VALUES (?,'title','desc','Aangenomen','2026')
        RETURNING id
        "#,
        random_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .id
}
