mod components;

use components::card::*;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use reqwest::get;
use shared::{MotieDto, VoteDto, GET_FIRST_MOTIE};
const BASE_URL: &str = "http://localhost:3000";

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
#[component]
fn MotionView() -> Element {
    let mut motion = use_resource(|| async move {
        reqwest::get(&format!("{}{}", BASE_URL, GET_FIRST_MOTIE))
            .await
            .unwrap()
            .json::<MotieDto>()
            .await
            .unwrap()
    });
    tracing::info!("{:?}", motion);
    let content = motion.value().with(|opt| {
        if let Some(m) = opt {
            rsx! {
            Card {
                CardHeader {
                    CardTitle { "{m.title}" }
                }
                CardContent {
                    p { "{m.description}" }
                }
                CardFooter {
                    button { onclick: move |_| motion.restart(), "not interested" }
                    button { onclick: move |_| motion.restart(), "vote!" }
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
