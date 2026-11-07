use std::path::PathBuf;

use crate::{
    components::{
        icon::{Icon, IconName},
        modal::{Modal, ModalWidth},
        ui::{Button, IconButton, Variant},
    },
    model::app_state::AppState,
};

use dioxus::prelude::*;
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
pub fn NewProjectModal(open: Signal<bool>, on_created: EventHandler<()>) -> Element {
    let mut target_folder = use_signal(|| {
        let state = AppState::load_or_default();
        state
            .selected_folder
            .or_else(|| state.roots.first().cloned())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });
    let mut project_name = use_signal(|| "".to_string());
    let mut lib_source = use_signal(|| LibSource::CratesIO);
    let mut custom_path = use_signal(|| "".to_string());
    let mut plottery_home_path = use_signal(|| std::env::var("PLOTTERY_HOME").unwrap_or_default());
    let mut error = use_signal(|| "".to_string());

    let custom_path_missing =
        matches!(lib_source(), LibSource::Path { .. }) && custom_path.read().is_empty();
    let can_submit = !project_name.read().is_empty() && !custom_path_missing;
    let target_preview = PathBuf::from(target_folder())
        .join(project_name())
        .to_string_lossy()
        .to_string();

    let mut submit = move || {
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

        error.set("".to_string());

        let mut app_state = AppState::load_or_default();
        app_state.ensure_root_covers(&project.dir);
        app_state.selected_folder = project.dir.parent().map(|p| p.to_path_buf());
        app_state.save();

        open.set(false);
        on_created.call(());
    };

    rsx! {
        Modal {
            open,
            width: ModalWidth::Form,
            title: "New project",
            subtitle: "Creates a Cargo project from the plottery template.",
            body: rsx! {
                style { {include_str!("./project_create.css")} }
                div {
                    class: "new_project_form",
                    onkeydown: move |event| {
                        if event.key() == Key::Enter && can_submit {
                            event.prevent_default();
                            submit();
                        }
                    },
                    label { class: "flabel",
                        "Name"
                        input {
                            class: "input lg",
                            name: "name",
                            autofocus: true,
                            required: true,
                            value: "{project_name}",
                            placeholder: "my_project",
                            oninput: move |event| project_name.set(event.value()),
                        }
                    }
                    div { class: "flabel",
                        label { r#for: "new_project_folder", "Location" }
                        div { class: "hrow",
                            input {
                                id: "new_project_folder",
                                class: "input lg",
                                name: "folder",
                                required: true,
                                value: target_folder,
                                placeholder: "target folder",
                                oninput: move |event| target_folder.set(event.value()),
                            }
                            IconButton {
                                icon: IconName::Folder,
                                label: "Choose folder",
                                variant: Variant::Secondary,
                                onclick: move |_| {
                                    if let Some(path) = FileDialog::new().pick_folder() {
                                        target_folder.set(path.to_string_lossy().to_string());
                                    }
                                },
                            }
                        }
                        span { class: "hint mono selectable", "→ {target_preview}" }
                    }
                    label { class: "flabel",
                        "Library source"
                        select {
                            class: "input lg select",
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
                            option { value: "cratesio", "crates.io (default)" }
                            option { value: "home", "PLOTTERY_HOME" }
                            option { value: "path", "Custom path" }
                        }
                    }
                    if matches!(lib_source(), LibSource::Path { .. }) {
                        label { class: "flabel",
                            "Path to plottery"
                            input {
                                class: if custom_path_missing { "input lg invalid" } else { "input lg" },
                                "aria-invalid": if custom_path_missing { "true" } else { "false" },
                                value: "{custom_path}",
                                placeholder: "/path/to/plottery",
                                oninput: move |event| {
                                    let path = event.value();
                                    custom_path.set(path.clone());
                                    lib_source.set(LibSource::Path { path: PathBuf::from(path) });
                                },
                            }
                            if custom_path_missing {
                                span { class: "hint field_error",
                                    Icon { name: IconName::Alert }
                                    "Path is required when the library source is set to Custom path."
                                }
                            }
                        }
                    }
                    if !error.read().is_empty() {
                        p { class: "field_error form_error", role: "alert",
                            Icon { name: IconName::Alert }
                            "{error}"
                        }
                    }
                }
            },
            footer: rsx! {
                Button {
                    variant: Variant::Ghost,
                    onclick: move |_| open.set(false),
                    "Cancel"
                }
                Button {
                    variant: Variant::Primary,
                    disabled: !can_submit,
                    onclick: move |_| submit(),
                    "Create project"
                }
            },
        }
    }
}
