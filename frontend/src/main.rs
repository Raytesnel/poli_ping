use dioxus::logger::tracing;
use dioxus::prelude::*;
use reqwest::get;
use shared::{GET_FIRST_MOTIE, MotieDto, VoteDto};
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
    tracing::info!("{:?}",motion);
    rsx! {
        div { id: "motion_view",
                {motion.value().cloned().map(|m| m.title).unwrap_or_else(|| "Loading...".to_string())}
        }
        div { id: "buttons",
            button { onclick: move |_| motion.restart(), id: "skip", "not intereset" }
            button { onclick: move |_| motion.restart(), id: "save", "vote!" }
        }
    }
}
