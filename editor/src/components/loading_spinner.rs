use dioxus::prelude::*;

#[component]
pub fn Loading() -> Element {
    rsx!(
        style { { include_str!("./loading_spinner.css") } }
        div { class: "LoadingSpinner",
            div { class: "spinner_item" }
        }
    )
}
