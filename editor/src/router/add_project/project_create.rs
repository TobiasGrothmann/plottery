use std::path::PathBuf;

use crate::{model::app_state::AppState, util::format_svg};

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::{LibSource, Project};
use rfd::FileDialog;

#[component]
pub fn ProjectCreate() -> Element {
    let mut target_folder = use_signal(|| "".to_string());
    let mut project_name = use_signal(|| "".to_string());
    let mut error = use_signal(|| "".to_string());

    rsx! {
        style { { include_str!("./project_create.css") } }
        div { class: "ProjectCreate",
            p { "Target folder" }
            input {
                name: "folder",
                style: "flex: 1;",
                required: true,
                value: target_folder,
                placeholder: "target folder",
                onchange: move |event| target_folder.set(event.value())
            }
            button { class: "folder_button",
                onclick: move |_event| {
                    let path = FileDialog::new()
                    .pick_folder();
                    if let Some(path) = path {
                        target_folder.set(path.to_string_lossy().to_string());
                    }
                },
                img { class: "folder_img",
                    src: "{format_svg(include_bytes!(\"../../../public/icons/folder_open.svg\"))}"
                },
            }

            p { "Project name" }
            input {
                name: "name",
                style: "flex: 1;",
                required: true,
                value: "{project_name}",
                placeholder: "project name",
                onchange: move |event| project_name.set(event.value()),
                grid_column: "span 2",
            }

            if !error.read().is_empty() {
                div { class: "err_box",
                    grid_column: "span 3",
                    p { "{error}" }
                }
            }

            button { class: "img_button accept",
                grid_column: "span 3",
                onclick: move |_event| {
                    if target_folder.read().is_empty() {
                        error.set("Please pick a target folder.".to_string());
                        return;
                    }
                    let folder = PathBuf::from(target_folder.read().clone());
                    if !folder.exists() {
                        error.set("Folder does not exist".to_string());
                        return;
                    }

                    let name: String = project_name.read().clone();
                    if name.contains(' ') {
                        error.set("Invalid project name - spaces are not allowed.".to_string());
                        return;
                    }

                    let re = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
                    if !re.is_match(&name) {
                        error.set("Invalid project name".to_string());
                        return;
                    }

                    let project = Project::new(folder, &name);
                    if let Err(e) = project.generate_to_disk(LibSource::Cargo) {
                        error.set(e.to_string());
                        return;
                    }

                    let app_state = AppState::load();
                    if app_state.is_none() {
                        error.set("Failed to load app state.".to_string());
                        return;
                    }
                    let mut app_state = app_state.unwrap();

                    // no error
                    error.set("".to_string());

                    app_state.projects.push(project);
                    app_state.save();

                    let nav = use_navigator();
                    nav.go_back();
                },
                img { src: "{format_svg(include_bytes!(\"../../../public/icons/check.svg\"))}" },
            }
        }
    }
}
