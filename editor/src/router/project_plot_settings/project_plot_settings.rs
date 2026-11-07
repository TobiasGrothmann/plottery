use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;

use crate::routes::Route;

#[component]
pub fn ProjectPlotSettings(project_path: Vec<String>) -> Element {
    let nav = use_navigator();
    use_effect(move || {
        nav.replace(Route::Editor {
            project_path: project_path.clone(),
        });
    });
    rsx! {}
}
