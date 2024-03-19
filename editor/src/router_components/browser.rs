use crate::components::navigation::Navigation;
use crate::util::format_svg;
use crate::{components::project_list::ProjectList, model::app_state::AppState, routes::Route};
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::Project;

#[component]
pub fn Browser(cx: Scope) -> Element {
    let app_state = use_state(cx, || {
        AppState::load().unwrap_or_else(|| {
            log::info!("App state file does not exist. Creating new app state.");
            let new_state = AppState::new();
            new_state.save();
            new_state
        })
    });

    cx.render(rsx! {
        style { include_str!("./browser.css") }
        Navigation { page_name: "Projects" }
        div { class: "Browser",
            ProjectList {
                app_state: app_state.get().clone(),
                on_delete_clicked: move |project: Project| {
                    let mut app_state_deref = app_state.get().to_owned();
                    app_state_deref.projects.retain(|p| *p != project);
                    app_state_deref.save();
                    app_state.set(app_state_deref);
                }
            }
            button { class: "img-button",
                onclick: move |_event| {
                    use_navigator(cx).push(Route::ProjectCreate {});
                },
                img { src: "{format_svg(include_bytes!(\"../../public/icons/add.svg\"))}" }
            }
        }
    })
}
