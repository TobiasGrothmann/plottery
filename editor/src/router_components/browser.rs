use crate::{components::project_list::ProjectList, model::app_state::AppState};
use dioxus::prelude::*;
use plottery_project::Project;
use std::path::PathBuf;

#[component]
pub fn Browser(cx: Scope) -> Element {
    let mut app_state = AppState::load().unwrap_or_else(|| {
        log::info!("App state file does not exist. Creating new app state.");
        let new_state = AppState::new();
        new_state.save();
        new_state
    });

    app_state.projects.push(
        Project::load_from_file(PathBuf::from(
            "/Users/admin/Dropbox/rust/plottery/project/test/test_project/test_project.plottery",
        ))
        .unwrap(),
    );
    // app_state.save();

    cx.render(rsx! {
        ProjectList {
            app_state: app_state.clone(),
        }
    })
}
