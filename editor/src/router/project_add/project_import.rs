use crate::{model::app_state::AppState, util::format_svg};

use dioxus::prelude::*;
use dioxus_router::{hooks::use_navigator, prelude::Navigator};
use plottery_project::Project;
use rfd::FileDialog;

fn pick_project(error: &mut Signal<String>, navigator: &Navigator) {
    let path = FileDialog::new()
        .set_title("pick existing plottery project")
        .add_filter("plottery project", &["plottery"])
        .pick_file();
    let path = match path {
        Some(path) => path,
        None => return,
    };

    if !path.exists() {
        error.set("File does not exist".to_string());
        return;
    }

    let project_to_import = Project::load_from_file(path);
    let project_to_import = match project_to_import {
        Ok(project) => project,
        Err(e) => {
            error.set(e.to_string());
            return;
        }
    };

    let app_state = AppState::load();
    if app_state.is_none() {
        error.set("Failed to load app state.".to_string());
        return;
    }
    let mut app_state = app_state.unwrap();

    // no error
    error.set("".to_string());

    if app_state.projects.contains(&project_to_import) {
        error.set("Project already in Database. It does not need to be imported.".to_string());
        return;
    }

    app_state.projects.push(project_to_import);
    app_state.save();

    navigator.go_back();
}

#[component]
pub fn ProjectImport() -> Element {
    let mut error = use_signal(|| "".to_string());
    let navigator = use_navigator();

    rsx! {
        style { { include_str!("./project_import.css") } }
        div { class: "ProjectImport",
            div { class: "input_row",
                p { "Pick existing Plottery project " }
                button { class: "img_button",
                    onclick: move |_event| {
                        pick_project(&mut error, &navigator)
                    },
                    img {
                        src: "{format_svg(include_bytes!(\"../../../public/icons/folder_open.svg\"))}"
                    },
                }
            }

            if !error.read().is_empty() {
                div { class: "err_box",
                    p { "{error}" }
                }
            }
        }
    }
}
