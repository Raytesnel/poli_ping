pub const GET_MOTIES: &str = "/get_moties";
pub const GET_NEXT_MOTIE: &str = "/get_first_motie";
pub const GET_MOTIE_PROGRESS: &str = "/get_motie_progress";
pub const POST_USER_VOTE: &str = "/post_user_vote";
pub const BASE_URL_BACKEND: &str = "127.0.0.1:3000";

const _MOTIE_FETCH_URL: &str = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Zaak?$filter=Verwijderd%20eq%20false%20and%20Soort%20eq%20'Motie'and%20ApiGewijzigdOp%20gt%20{date}&$orderby=GewijzigdOp%20desc&$expand=Besluit($expand=Stemming($expand=Fractie)),Document";
const _GET_MOTIE_DOCUMENT: &str = "https://gegevensmagazijn.tweedekamer.nl/OData/v4/2.0/Document(7a9b77f1-d230-4a00-9856-2f0f8e967e62)/resource om document te zien";
