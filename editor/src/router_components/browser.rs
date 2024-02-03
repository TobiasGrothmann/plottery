use crate::{components::project_list::ProjectList, model::app_state::AppState, routes::Route};
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

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
            h1 { "Projects" }
            ProjectList {
                app_state: app_state.clone(),
            }
            button { class: "img-button",
                onclick: move |_event| {
                    let nav = use_navigator(cx);
                    nav.push(Route::ProjectCreate {});
                },
                img { src: "icons/add.svg" },
            }
        }
    })
}
