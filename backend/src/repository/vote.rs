use sqlx::SqlitePool;

pub async fn insert_user_vote(
    pool: &SqlitePool,
    user_id: &str,
    motie_id: i32,
    vote: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
            r#"
        INSERT INTO user_votes (user_id, motie_id, vote)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, motie_id)
        DO UPDATE SET vote = excluded.vote
        "#,
        user_id,
        motie_id,
        vote
    )
        .execute(pool)
        .await?;

    Ok(())
}