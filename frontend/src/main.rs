mod components;

use components::card::*;
use dioxus::logger::tracing::{event, span, Level};
use dioxus::prelude::*;
use reqwest::Client;
use shared::{AddUserVoteRequest, MotieDto, NextMotieRequest, BASE_URL_BACKEND, GET_NEXT_MOTIE, POST_USER_VOTE};

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
#[component]
fn MotionView() -> Element {
    let client = use_signal(Client::new);
    let motion_resource = use_resource(|| async { fetch_motion().await });
    let client_handle = client.read().clone();

    // helper function for voting
    let vote_and_refresh = {
        let motion_resource = motion_resource.clone(); // clone for closure
        move |motie_id: i32, vote_value: &'static str| {
            let client = client_handle.clone();
            let mut motion_resource = motion_resource.clone(); // clone for async
            spawn(async move {
                send_vote(client, motie_id, vote_value).await;
                println!("motie id {} sent vote", motie_id);
                motion_resource.restart();
            });
        }
    };

    let content = motion_resource.value().with(|maybe_result| {
        if let Some(result) = maybe_result {
            match result {
                Ok(motion) => {
                    let motie_id = motion.id;
                    rsx! {
                        MotionCard {
                            motion: motion.clone(),
                            on_vote: move |vote_value| vote_and_refresh(motie_id, vote_value),
                        }
                    }
                }
                Err(_) => rsx!(div { "Failed to fetch motion." }),
            }
        } else {
            rsx!(div { "Loading..." })
        }
    });

    rsx!(div {id: "motion_view", {content} })
}

// Separate MotionCard component
#[component]
fn MotionCard(motion: MotieDto, on_vote: EventHandler<&'static str>) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { "{motion.title}" }
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

// VoteButton component
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

// Modular async function for sending vote
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

// Modular fetch_motion
async fn fetch_motion() -> Result<MotieDto, reqwest::Error> {
    event!(Level::INFO, "Fetching motion");
    let json_request = NextMotieRequest {
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