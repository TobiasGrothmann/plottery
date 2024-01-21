use crate::{components::project_list::ProjectList, model::app_state::AppState};
use dioxus::prelude::*;

#[component]
pub fn Browser(cx: Scope) -> Element {
    let app_state = AppState::load().unwrap_or_else(|| {
        log::info!("App state file does not exist. Creating new app state.");
        let new_state = AppState::new();
        new_state.save();
        new_state
    });

    cx.render(rsx! {
        style { include_str!("./browser.css") }
        div { class: "Browser",
            ProjectList {
                app_state: app_state.clone(),
            }
        }
    })
}
