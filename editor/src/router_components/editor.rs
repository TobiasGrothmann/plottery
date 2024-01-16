use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::Project;

#[component]
pub fn Editor(cx: Scope, project_path: String) -> Element {
    let project = use_state(cx, || {
        let path = PathBuf::from(project_path.clone());
        Project::load_from_file(path).unwrap()
    });

    cx.render(rsx! {
        button {
            onclick: move |_event| {
                let nav = use_navigator(cx);
                nav.go_back();
            },
            "<-"
        }
        h1 {
            "{project.config.name}"
        }
    })
}
