use crate::models::api_models::MotieTransformed;
use crate::models::db_models::{Motie, MotieDocument, PartyVote};
use sqlx::SqlitePool;
use sqlx::{QueryBuilder, Row, Sqlite};

pub async fn get_next_unseen_motie(
    pool: &SqlitePool,
    user_id: &str,
) -> Result<Option<Motie>, sqlx::Error> {
    sqlx::query_as!(
        Motie,
        r#"
        SELECT id, external_id, title, description, result, timestamp
        FROM moties
        WHERE id NOT IN (
            SELECT motie_id
            FROM user_votes
            WHERE user_id = ?
        )
        ORDER BY id
        LIMIT 1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_party_votes(
    pool: &SqlitePool,
    motie_id: i64,
) -> Result<Vec<PartyVote>, sqlx::Error> {
    sqlx::query_as!(
        PartyVote,
        r#"
        SELECT id,motie_id,party, vote
        FROM party_votes
        WHERE motie_id = ?
        "#,
        motie_id
    )
    .fetch_all(pool)
    .await
}

pub async fn insert_motie(pool: &SqlitePool, motie: &MotieTransformed) -> Result<i64, sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO moties
    (external_id, title, description, result, timestamp)
    VALUES (?, ?, ?, ?, ?)
    ON CONFLICT(external_id) DO UPDATE SET external_id=excluded.external_id
    RETURNING id
    "#,
        motie.external_id,
        motie.title,
        motie.description,
        motie.result,
        motie.timestamp,
    )
    .fetch_one(pool)
    .await?;

    let record = sqlx::query!(
        r#"
        SELECT id FROM moties WHERE external_id = ?
        "#,
        &motie.external_id
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id.expect("REASON"))
}

pub async fn insert_party_vote(
    pool: &SqlitePool,
    motie_id: i64,
    party: &str,
    vote: &str,
) -> Result<(), sqlx::Error> {
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
    .await?;

    Ok(())
}

pub async fn existing_ids(pool: &SqlitePool, ids: &[String]) -> Result<Vec<String>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let mut qb =
        QueryBuilder::<Sqlite>::new("SELECT external_id FROM moties WHERE external_id IN (");

    let mut separated = qb.separated(",");

    for id in ids {
        separated.push_bind(id);
    }

    qb.push(")");

    let rows = qb.build().fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|row| row.get::<String, _>("external_id"))
        .collect())
}

pub async fn insert_documents(
    document_ids: &String,
    motie_id: i64,
    pool: &SqlitePool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
            r#"
        INSERT INTO motie_documents (motie_id, document_id)
        VALUES (?, ?)
        "#,
            motie_id,
            document_ids
        )
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_document_ids(pool: &SqlitePool, motie_id: &i64) -> Result<Vec<MotieDocument>, sqlx::Error> {
    sqlx::query_as!(
        MotieDocument,
        r#"
        SELECT id,motie_id,document_id
        FROM motie_documents
        WHERE motie_id = ?
        "#,
        motie_id
    )
        .fetch_all(pool)
        .await
}
