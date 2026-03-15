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

pub async fn get_document_ids(
    pool: &SqlitePool,
    motie_id: &i64,
) -> Result<Vec<MotieDocument>, sqlx::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{MotieDocumentDto, VoteDto};
    use tower_http::follow_redirect::policy::PolicyExt;
    use uuid::Uuid;
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

    async fn seed_vote(pool: &SqlitePool, motie_id: i32, party: &str, vote: &str) {
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
    async fn seed_document(pool: &SqlitePool, motie_id: i64, document_id: &str) {
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

    #[sqlx::test]
    async fn insert_motie_should_store_motie(pool: SqlitePool) {
        let documents = vec![MotieDocumentDto {
            document_id: "abc".into(),
        }];
        let votes = vec![
            VoteDto {
                party: "D66".into(),
                vote: "tegen".into(),
            },
            VoteDto {
                party: "VVD".into(),
                vote: "voor".into(),
            },
        ];
        let motie = MotieTransformed {
            external_id: "api-123".into(),
            title: "Test Motie".into(),
            description: "Description".into(),
            result: "Aangenomen".into(),
            timestamp: "2026-02-05".into(),
            votes: votes,
            documents: documents,
        };
        // Act
        let id = insert_motie(&pool, &motie).await.unwrap();

        let stored = sqlx::query!("SELECT title FROM moties WHERE id = ?", id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(stored.title, "Test Motie");
    }
    #[sqlx::test]
    async fn insert_party_vote_should_store_vote(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let party = "D66";
        let vote = "tegen";

        // Then
        insert_party_vote(&pool, motie_id.clone(), party, vote)
            .await
            .unwrap();

        // When
        let stored = sqlx::query_as!(
            PartyVote,
            "SELECT id,party,vote,motie_id FROM party_votes WHERE motie_id = ?",
            motie_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(stored.party, party);
        assert_eq!(stored.vote, vote);
        assert_eq!(stored.motie_id, motie_id);
    }
    #[sqlx::test]
    async fn insert_document_should_store_vote(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let document_id = "some_id".to_string();

        // When
        insert_documents(&document_id, motie_id.clone(), &pool)
            .await
            .unwrap();

        // Then
        let stored = sqlx::query_as!(
            MotieDocument,
            "SELECT id,motie_id,document_id FROM motie_documents WHERE motie_id = ?",
            motie_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(stored.motie_id, motie_id);
        assert_eq!(stored.document_id, document_id);
    }
    #[sqlx::test]
    async fn get_document_ids_of_given_motie(pool: SqlitePool) {
        let motie_id = seed_motie(&pool).await;
        let motie_id_2 = seed_motie(&pool).await;
        seed_document(&pool, motie_id, "abc").await;
        seed_document(&pool, motie_id, "def").await;
        seed_document(&pool, motie_id_2, "ghi").await;

        let expected_document_ids = vec!["abc", "def"];
        // Act
        let result = get_document_ids(&pool, &motie_id).await.unwrap();

        assert_eq!(
            result
                .iter()
                .map(|id| id.document_id.clone())
                .collect::<Vec<_>>(),
            expected_document_ids
        );
    }

    #[sqlx::test]
    async fn get_next_unseens_motie_when_no_vote(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let motie_id_2 = seed_motie(&pool).await;
        let user = "Ash";

        // When
        let result_moties = get_next_unseen_motie(&pool, user).await;

        // Then
        assert_eq!(result_moties.unwrap().unwrap().id, motie_id);
    }

    #[sqlx::test]
    async fn get_next_unseens_motie_when_user_voted(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let motie_id_2 = seed_motie(&pool).await;
        let motie_id_3 = seed_motie(&pool).await;
        let user = "Ash";
        sqlx::query!(
            r#"
        INSERT INTO user_votes (user_id, motie_id, vote)
        VALUES (?, ?, ?)
        ON CONFLICT(user_id, motie_id)
        DO UPDATE SET vote = excluded.vote
        "#,
            user,
            motie_id,
            "voor"
        )
        .execute(&pool)
        .await
        .unwrap();

        // When
        let result_moties = get_next_unseen_motie(&pool, user).await;

        // Then
        assert_eq!(result_moties.unwrap().unwrap().id, motie_id_2);
    }

    #[sqlx::test]
    async fn get_votes_from_user(pool: SqlitePool) {
        // Given
        let motie_id = seed_motie(&pool).await;
        let party = "D66";
        seed_vote(&pool, motie_id as i32, &party, "tegen").await;
        seed_vote(&pool, motie_id as i32, &party, "voor").await;
        // Then
        let result_votes = get_party_votes(&pool, motie_id.clone()).await.unwrap();

        // When
        for result in &result_votes {
            assert_eq!(result.party, party);
            assert_eq!(result.motie_id, motie_id);
        }
        assert_eq!(result_votes.len(), 2);
        assert!(result_votes.iter().any(|v| v.vote == "tegen"));
        assert!(result_votes.iter().any(|v| v.vote == "voor"));
    }

    #[sqlx::test]
    async fn existing_ids_should_return_only_existing(pool: SqlitePool) {
        // Given
        sqlx::query!(
            "INSERT INTO moties (external_id,title,description,result,timestamp)
         VALUES ('existing-id','t','d','r','t')"
        )
        .execute(&pool)
        .await
        .unwrap();
        let ids = vec!["existing-id".to_string(), "not_existing_id".to_string()];


        let result = existing_ids(&pool, &ids).await.unwrap();

        assert_eq!(result, vec!["existing-id"]);
    }
}
