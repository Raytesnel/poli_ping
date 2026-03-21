/* tests

given the state of 2 moties
when the user ask for next motie
then the user should see the latest not voted motie

given the state of 2 moties
when the user votes on the seen motie
then the next not voted motie will be shown.

given the state of 1 unvoted motie
when the user votes
then the message 'no more moties' will be shown

given the state of 3 unvoted moties and 10 total moties
when the asks for progress
then the user gets an overview of 3:10

given the state of 3 unvoted moties and 10 total moties
when the asks for progress after a vote
then the user gets an overview of 4:10

given the state of 0 unvoted moties
when the user asks for next unvoted motie
then the message 'no more moties' will be shown

given 2 moties
when a new user is used
then progress should be 0:2

*/
mod helpers;
use sqlx::SqlitePool;
use helpers::{test_server, seed_motie};
use shared::{MotieDto, GET_NEXT_MOTIE};


///Given the state of 1 unvoted motie
///When asks the next unvoted motie
///Then the motie contains at least:
///     title
///     description
///     votes
///     maker (person & party)
///     when made
///     more info (document id)
#[sqlx::test]
async fn get_next_motie_returns_first_unvoted(pool: SqlitePool) {
    // Given
    let motie_id = seed_motie(&pool).await;
    let server = test_server(pool);

    // When
    let response = server
        .post(GET_NEXT_MOTIE)
        .json(&serde_json::json!({
            "user_id": "user1"
        }))
        .await;

    // Then
    response.assert_status_ok();

    let body: MotieDto = response.json();
    assert_eq!(body.id, motie_id as i32);
}
