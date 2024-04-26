use dioxus::prelude::*;

#[component]
pub fn Loading() -> Element {
    rsx!(
        style { { include_str!("./loading.css") } }
        div { class: "Loading",
            div { class: "spinner_item" }
        }
    )
}
