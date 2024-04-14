use crate::components::navigation::Navigation;
use crate::util::format_svg;
use crate::{components::project_list::ProjectList, model::app_state::AppState, routes::Route};
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::Project;

#[component]
pub fn Browser() -> Element {
    let mut app_state = use_signal(|| {
        AppState::load().unwrap_or_else(|| {
            log::info!("App state file does not exist. Creating new app state.");
            let new_state = AppState::new();
            new_state.save();
            new_state
        })
    });

    rsx! {
        style { { include_str!("./browser.css") } }
        Navigation { page_name: "Projects" }
        div { class: "Browser",
            ProjectList {
                app_state: app_state,
                on_delete_clicked: move |project: Project| {
                    let mut new_app_state = app_state.read().clone();
                    new_app_state.projects.retain(|p| *p != project);
                    new_app_state.save();
                    app_state.set(new_app_state);
                }
            }
            button { class: "img_button",
                onclick: move |_event| {
                    use_navigator().push(Route::ProjectAdd {});
                },
                img { src: "{format_svg(include_bytes!(\"../../public/icons/add.svg\"))}" }
            }
        }
    }
}
