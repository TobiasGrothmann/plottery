use std::path::PathBuf;

use crate::{model::app_state::AppState, util::format_svg};

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_project::{LibSource, Project};
use rfd::FileDialog;

fn validate_form(
    target_folder: &str,
    project_name: &str,
    lib_source: &LibSource,
    custom_path: &str,
    plottery_home_path: &str,
) -> Result<(), String> {
    if target_folder.is_empty() {
        return Err("Please pick a target folder.".to_string());
    }

    if !PathBuf::from(target_folder).exists() {
        return Err("Target folder does not exist".to_string());
    }

    if project_name.contains(' ') {
        return Err("Invalid project name - spaces are not allowed.".to_string());
    }

    let re = regex::Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    if !re.is_match(project_name) {
        return Err("Invalid project name".to_string());
    }

    match lib_source {
        LibSource::PlotteryHome => {
            if plottery_home_path.is_empty() {
                return Err("PLOTTERY_HOME environment variable is not set".to_string());
            }
            if !PathBuf::from(plottery_home_path).exists() {
                return Err(format!(
                    "PLOTTERY_HOME path does not exist: {}",
                    plottery_home_path
                ));
            }
        }
        LibSource::Path { path } => {
            if custom_path.is_empty() {
                return Err(
                    "Custom path is required when the library source is set to \"Custom Path\""
                        .to_string(),
                );
            }
            if !path.exists() {
                return Err(format!("Custom path does not exist: {}", custom_path));
            }
        }
        LibSource::CratesIO => {}
    }

    Ok(())
}

#[component]
pub fn ProjectCreate() -> Element {
    let mut target_folder = use_signal(|| "".to_string());
    let mut project_name = use_signal(|| "".to_string());
    let mut lib_source = use_signal(|| LibSource::CratesIO);
    let mut custom_path = use_signal(|| "".to_string());
    let mut plottery_home_path = use_signal(|| std::env::var("PLOTTERY_HOME").unwrap_or_default());
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

            p { "Library source" }
            select {
                style: "flex: 1;",
                value: match lib_source() {
                    LibSource::CratesIO => "cratesio",
                    LibSource::PlotteryHome => "home",
                    LibSource::Path { .. } => "path",
                },
                onchange: move |event| {
                    match event.value().as_str() {
                        "cratesio" => lib_source.set(LibSource::CratesIO),
                        "home" => {
                            let home_path = std::env::var("PLOTTERY_HOME").unwrap_or_default();
                            plottery_home_path.set(home_path);
                            lib_source.set(LibSource::PlotteryHome);
                        },
                        "path" => lib_source.set(LibSource::Path { path: PathBuf::from(custom_path()) }),
                        _ => {}
                    }
                },
                grid_column: "span 2",
                option { value: "cratesio", "crates.io (default)" }
                option { value: "home", "$PLOTTERY_HOME env var" }
                option { value: "path", "custom path" }
            }

            if matches!(lib_source(), LibSource::Path { .. }) {
                p { "Custom path" }
                input {
                    style: "flex: 1;",
                    value: "{custom_path}",
                    placeholder: "/path/to/plottery",
                    onchange: move |event| {
                        let path = event.value();
                        custom_path.set(path.clone());
                        lib_source.set(LibSource::Path { path: PathBuf::from(path) });
                    },
                    grid_column: "span 2",
                }
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
                    if let Err(validation_error) = validate_form(
                        &target_folder.read(),
                        &project_name.read(),
                        &lib_source(),
                        &custom_path.read(),
                        &plottery_home_path.read(),
                    ) {
                        error.set(validation_error);
                        return;
                    }

                    let folder = PathBuf::from(target_folder.read().clone());
                    let project = Project::new(folder, &project_name.read());
                    if let Err(e) = project.generate_to_disk(lib_source(), true) {
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
