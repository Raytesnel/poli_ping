mod components;

use components::card::*;
use dioxus::logger::tracing::{event, span, Level};
use dioxus::prelude::*;
use reqwest::Client;
use shared::{AddUserVoteRequest, MotieDto, MotieProgressDto, UserIdRequest, BASE_URL_BACKEND, GET_MOTIE_PROGRESS, GET_NEXT_MOTIE, POST_USER_VOTE};

const USER_ID: &str = "dev-user_2";
fn main() {
    let root_span = span!(Level::INFO, "frontend_startup");
    let _guard = root_span.enter();

    event!(Level::INFO, "Launching Dioxus app");
    dioxus::launch(App);
}
static CSS: Asset = asset!("/assets/main.css");

#[component]
fn App() -> Element {
    rsx! {
        Stylesheet { href: CSS }
        Title {}
        MotionView {}
    }
}

#[component]
fn Title() -> Element {
    rsx! {
        div { id: "title",
            h1 { "Polling time!" }
        }
    }
}

/// Component that renders the current motion (motie) and its voting progress.
///
/// This component fetches both the next unseen motion for the user and
/// the voting progress (how many motions the user has already voted on).
///
/// While loading, it shows a "Loading..." message. On error, it logs it and
/// displays a relevant message. Once both motion and progress are available,
/// it displays a `MotionCard` with the motion details and vote buttons.
#[component]
fn MotionView() -> Element {
    let client = use_signal(Client::new);
    let motion_resource = use_resource(|| async { fetch_motion().await });
    let progress_state = use_resource(|| async { fetch_motie_progress().await });
    let client_handle = client.read().clone();

    // helper function for voting
    let vote_and_refresh = {
        let motion_resource = motion_resource.clone(); // clone for closure
        let progress_state = progress_state.clone(); // clone for closure
        move |motie_id: i32, vote_value: &'static str| {
            let client = client_handle.clone();
            let mut motion_resource = motion_resource.clone(); // clone for async
            let mut progress_state = progress_state.clone(); // clone for async
            spawn(async move {
                send_vote(client, motie_id, vote_value).await;
                info!("motie id {} sent vote", motie_id);
                motion_resource.restart();
                progress_state.restart();
            });
        }
    };
    let content = motion_resource.value().with(|maybe_motion| {
        progress_state.value().with(|maybe_progress| {
            match (maybe_motion, maybe_progress) {
                (Some(Ok(motion)), Some(Ok(progress))) => {
                    let motie_id = motion.id;
                    rsx! {
                    MotionCard {
                        motion: motion.clone(),
                        progress: progress.clone(),
                        on_vote: move |vote_value| vote_and_refresh(motie_id, vote_value),
                    }
                }
                }

                (Some(Err(e)), _) => {
                    event!(Level::ERROR,"Motion error: {:?}", e);
                    rsx!(div { "Failed to fetch motion." })
                }

                (_, Some(Err(e))) => {
                    event!(Level::ERROR,"Progress error: {:?}", e);
                    rsx!(div { "Failed to fetch progress." })
                }

                _ => rsx!(div { "Loading..." }),
            }
        })
    });
    rsx!(div {id: "motion_view", {content} })
}

/// Card component that displays a motion with title, description,
/// voting buttons, and current voting progress.
///
/// # Props
/// - `motion`: The motion data (`MotieDto`) to display.
/// - `progress`: The voting progress for the user (`MotieProgressDto`).
/// - `on_vote`: Callback to trigger when the user votes.
#[component]
fn MotionCard(motion: MotieDto, on_vote: EventHandler<&'static str>,progress:MotieProgressDto) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{motion.title}" }
                p { "{progress.voted}/{progress.total} voted" }
            }
            CardContent {
                p { "{motion.description}" }
            }
            CardFooter {
                VoteButton { label: "Voor", value: "VOOR", on_vote }
                VoteButton { label: "Tegen", value: "TEGEN", on_vote }
                VoteButton { label: "Niet interessant", value: "NIET_INTERESSANT", on_vote }
            }
        }
    }
}

/// Card component that creates a vote button,
///
/// # Props
/// - `label`: The text to display on the button ('voor', 'tegen', 'niet interesant').
/// - `value`: The value to save ('voor', 'tegen', 'niet interesant').
/// - `on_vote`: Callback to trigger when the user votes.
#[component]
fn VoteButton(
    label: &'static str,
    value: &'static str,
    on_vote: EventHandler<&'static str>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| on_vote.call(value),
            "{label}"
        }
    }
}

/// Send a vote for a given motion.
///
/// Posts a vote to the backend using the `Client`.
/// Logs the event with tracing.
async fn send_vote(client: Client, motie_id: i32, vote_value: &str) {
    event!(Level::INFO, "Sending vote: {vote_value}");
    let vote = AddUserVoteRequest {
        user_id: USER_ID.to_string(),
        motie_id,
        vote: vote_value.to_string(),
    };

    let _ = client
        .post(&format!("http://{}{}", BASE_URL_BACKEND, POST_USER_VOTE))
        .json(&vote)
        .send()
        .await;
}

/// Fetch the next unseen motion for the current user.
///
/// Sends a POST request to the backend with the `USER_ID` and returns
/// a `MotieDto` if available.
///
/// # Errors
/// Returns a `reqwest::Error` if the HTTP request or JSON deserialization fails.
async fn fetch_motion() -> Result<MotieDto, reqwest::Error> {
    event!(Level::INFO, "Fetching motion");
    let json_request = UserIdRequest {
        user_id: USER_ID.to_string(),
    };
    let resp = Client::new()
        .post(&format!("http://{}{}", BASE_URL_BACKEND, GET_NEXT_MOTIE))
        .json(&json_request)
        .send()
        .await?;
    let motion = resp.json::<MotieDto>().await?;
    event!(Level::DEBUG, "motion: {:?}", motion);
    Ok(motion)
}

/// Fetch the current user's progress (how many motions voted).
///
/// Sends a POST request to the backend and returns a `MotieProgressDto`.
///
/// # Errors
/// Returns a `reqwest::Error` if the request or JSON deserialization fails.
async fn fetch_motie_progress() -> Result<MotieProgressDto, reqwest::Error> {
    let req = UserIdRequest {
        user_id: USER_ID.to_string(),
    };
    let resp = Client::new()
        .post(&format!("http://{}{}", BASE_URL_BACKEND, GET_MOTIE_PROGRESS))
        .json(&req)
        .send()
        .await?;
    let progress = resp.json::<MotieProgressDto>().await?;
    event!(Level::DEBUG, "progression: {:?}", progress);
    Ok(progress)
}