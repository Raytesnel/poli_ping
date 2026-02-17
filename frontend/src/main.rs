use dioxus::prelude::*;
use reqwest::get;
use shared::{MotieDto, GET_MOTIES};
const BASE_URL: &str = "/http://localhost:8080";

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
    let mut payload_moties = use_resource(|| async move {
        reqwest::get(&format!("{}/{}",BASE_URL,GET_MOTIES))
            .await
            .unwrap()
            .json::<MotieDto>()
            .await
            .unwrap()
    });

    rsx! {
        div { id: "motion_view",
            div { id: "motion",

            }
        }
        div { id: "buttons",
            button { onclick: move |_| payload_moties.restart(), id: "skip", "not intereset" }
            button { onclick: move |_| payload_moties.restart(), id: "save", "vote!" }
        }
    }
}





