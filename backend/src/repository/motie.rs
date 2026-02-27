use crate::models::api_models::MotieTransformed;
use crate::models::db_models::{Motie, PartyVote};
use sqlx::SqlitePool;

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
