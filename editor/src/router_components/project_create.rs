use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::Project;
use rfd::FileDialog;

#[component]
pub fn ProjectCreate(cx: Scope) -> Element {
    let target_folder = use_state(cx, || "".to_string());
    let project_name = use_state(cx, || "".to_string());

    let error = use_state(cx, || "".to_string());

    cx.render(rsx! {
        style { include_str!("./project_create.css") }

        div { class: "ProjectCreate",
            h1 { "Create Project" }

            div { class: "input-row",
                div { class: "folder-group",
                    input {
                        name: "folder",
                        style: "flex: 1;",
                        required: true,
                        value: target_folder.get().as_str(),
                        placeholder: "path/to/folder",
                        onchange: move |event| target_folder.set(event.value.clone())
                    }
                    button { class: "img-button",
                        onclick: move |_event| {
                            let path = FileDialog::new()
                            .set_directory("/")
                            .pick_folder();
                            if let Some(path) = path {
                                target_folder.set(path.to_string_lossy().to_string());
                            }
                        },
                        img { src: "icons/folder_open.svg" },
                    }
                }
                div { class: "input-row",
                    input {
                        name: "name",
                        required: true,
                        value: project_name.get().as_str(),
                        placeholder: "awesome_project",
                        onchange: move |event| project_name.set(event.value.clone())
                    }
                }

                if !error.get().is_empty() {
                    cx.render(rsx! {
                        div { class: "err_box",
                            p { error.get().clone() }
                        }
                    })
                }

                button { class: "img-button accept",
                    onclick: move |_event| {
                        let folder = PathBuf::from(target_folder.get());
                        if !folder.exists() {
                            error.set("Folder does not exist".to_string());
                            return;
                        }

                        let name = project_name.get();
                        let re = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
                        if !re.is_match(&name) {
                            error.set("Invalid project name".to_string());
                            return;
                        }

                        // no error
                        error.set("".to_string());

                        let project = Project::new(folder, name.to_string());
                        match project.generate_to_disk() {
                            Ok(_) => {
                                log::info!("Project created");
                                let nav = use_navigator(cx);
                                nav.go_back();
                            },
                            Err(e) => {
                                error.set(e.to_string());
                            }
                        }
                    },
                    img { src: "icons/check.svg" },
                }
            }
        }
    })
}
