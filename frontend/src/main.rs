mod components;

use components::card::*;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use reqwest::Client;
use reqwest::get;
use shared::{AddUserVoteRequest, GET_NEXT_MOTIE, MotieDto, POST_USER_VOTE, VoteDto};
const BASE_URL: &str = "http://127.0.0.1:3000";
const USER_ID: &str = "dev-user";
fn main() {
    dioxus::launch(App);
}
static CSS: Asset = asset!("/assets/main.css");

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: CSS }
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

async fn send_vote(client: Client, motie_id: i32, vote_value: &str) {
    let vote = AddUserVoteRequest {
        user_id: USER_ID.to_string(),
        motie_id,
        vote: vote_value.to_string(),
    };

    client
        .post(&format!("{}{}", BASE_URL, POST_USER_VOTE))
        .json(&vote)
        .send()
        .await
        .unwrap();
}

#[component]
fn MotionView() -> Element {
    let client = Client::new();
    let mut motion = use_resource(|| async move {
        get(&format!("{}{}", BASE_URL, GET_NEXT_MOTIE))
            .await
            .unwrap()
            .json::<MotieDto>()
            .await
            .unwrap()
    });
    let content = motion.value().with(|opt| {
        if let Some(m) = opt {
            let motie_id = m.id;
            let vote_button = |label: &str, vote_value: &'static str, client:Client| {
                rsx! {
                    button {
                        onclick: move |_| {
                            let value = client.clone();
                            spawn(async move {
                                send_vote(value, motie_id, vote_value).await;
                                motion.restart();
                            });
                        },
                        "{label}"
                    }
                }
            };
            rsx! {
            Card {
                CardHeader {
                    CardTitle { "{m.title}" }
                }
                CardContent {
                    p { "{m.description}" }
                }
                CardFooter {
                    {vote_button("Voor", "VOOR", client.clone())}
                    {vote_button("Tegen", "TEGEN", client.clone())}
                    {vote_button("Niet interessant", "NIET_INTERESSANT", client.clone())}
                }

                        }
                    }
        } else {
            rsx! {
                Card {
                    CardHeader {
                        CardTitle { "Loading..." }
                    }
                    CardContent {
                        p { "Fetching motion..." }
                    }
                }
            }
        }
    });

    rsx! { div { id: "motion_view",{content}} }
}
