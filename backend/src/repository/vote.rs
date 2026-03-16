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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use crate::models::db_models::UserVotes;

    async fn seed_motie(pool: &SqlitePool) -> i64 {
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
    #[sqlx::test]
    async fn save_user_votes_to_db(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let user_id = "Ash";
        let vote = "tegen";
        // When
        let result = insert_user_vote(&pool, &user_id, motie_id as i32, vote).await;
        // Then
        assert!(result.is_ok());
        let vote_result = sqlx::query_as!(
            UserVotes,
        r#"
        SELECT user_id,motie_id, vote
        FROM user_votes
        WHERE motie_id = ?
        "#,
        motie_id
    )
            .fetch_all(&pool)
            .await.unwrap();
        assert_eq!(vote_result.len(), 1);
        assert_eq!(vote_result.first().unwrap().user_id, user_id);
        assert_eq!(vote_result.first().unwrap().vote, vote);

    }

}
