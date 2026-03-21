use axum_test::TestServer;
use backend::{app::create_app};
use sqlx::SqlitePool;
use uuid::Uuid;

pub fn test_server(pool: SqlitePool) -> TestServer {
    let (app,state) = create_app(pool);
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

pub async fn seed_vote(pool: &SqlitePool, motie_id: i32, party: &str, vote: &str) {
    sqlx::query!(
            r#"
        INSERT OR IGNORE INTO party_votes
        (motie_id, party, vote)
        VALUES (?, ?, ?)
        "#,
            motie_id as i32,
            party,
            vote
        )
        .execute(pool)
        .await
        .unwrap();
}
pub async fn seed_document(pool: &SqlitePool, motie_id: i64, document_id: &str) {
    sqlx::query!(
            "INSERT INTO motie_documents (motie_id,document_id)
         VALUES (?,?)",
            motie_id,
            document_id
        )
        .execute(pool)
        .await
        .unwrap();
}