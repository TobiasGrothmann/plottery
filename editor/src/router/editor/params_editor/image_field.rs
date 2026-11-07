use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{file_name_of, project_param_value::ProjectParamValue, ProjectParamImage};
use rfd::FileDialog;

use crate::{
    router::editor::params_editor::{
        params_editor::LeafProps,
        widgets::{dropdown::Dropdown, segmented::Segmented},
    },
    util::image_import::{import_image, resolve_image_path, DOWNSCALE_MAX_PX},
};

fn edit_image(props: &LeafProps, edit: impl FnOnce(&mut ProjectParamImage)) {
    props.edit(|value| {
        if let ProjectParamValue::Image(image) = value {
            edit(image);
        }
    });
}

#[component]
pub fn ImageField(props: LeafProps) -> Element {
    let ProjectParamValue::Image(image) = &props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let mut copy_mode = use_signal(|| true);
    let mut downscale = use_signal(|| true);
    let mut import_error = use_signal_sync(|| None::<String>);

    let current = image.path.clone();
    let missing = current
        .as_deref()
        .map(|stored| !resolve_image_path(&props.project_dir, stored).exists())
        .unwrap_or(false);
    let shown_name = current
        .as_deref()
        .map(file_name_of)
        .unwrap_or_else(|| "no image".to_string());
    let recent = image.recent.clone();
    let recent_names: Vec<String> = recent.iter().map(|p| file_name_of(p)).collect();

    let choose_props = props.clone();
    let recent_props = props.clone();
    let clear_props = props.clone();
    let project_dir: PathBuf = props.project_dir.clone();

    rsx! {
        style { { include_str!("image_field.css") } }
        div { class: "ImageField",
            div { class: "image_row",
                Segmented {
                    options: vec!["copy".to_string(), "link".to_string()],
                    selected: Some(if copy_mode() { 0 } else { 1 }),
                    onselect: move |index: usize| copy_mode.set(index == 0),
                }
            }
            if copy_mode() {
                label { class: "image_downscale",
                    input {
                        r#type: "checkbox",
                        checked: downscale(),
                        onchange: move |event| downscale.set(event.value().parse().unwrap_or(true)),
                    }
                    "downscale copy to max. {DOWNSCALE_MAX_PX} px (longest side)"
                }
            }
            div { class: "image_row",
                button {
                    r#type: "button",
                    class: "image_button",
                    onclick: move |_| {
                        let Some(picked) = FileDialog::new()
                            .add_filter("images", &["png", "jpg", "jpeg"])
                            .pick_file()
                        else {
                            return;
                        };
                        import_error.set(None);
                        if !copy_mode() {
                            edit_image(&choose_props, |image| {
                                image.set(Some(picked.to_string_lossy().to_string()))
                            });
                            return;
                        }
                        let project_dir = project_dir.clone();
                        let props = choose_props.clone();
                        let downscale = downscale();
                        tokio::spawn(async move {
                            let result = tokio::task::spawn_blocking(move || {
                                import_image(&project_dir, &picked, downscale)
                            })
                            .await;
                            match result {
                                Ok(Ok(stored)) => edit_image(&props, |image| image.set(Some(stored))),
                                Ok(Err(e)) => {
                                    tracing::error!("Failed to import image: {:?}", e);
                                    if let Ok(mut err) = import_error.try_write() {
                                        *err = Some(format!("{e:#}"));
                                    }
                                }
                                Err(e) => tracing::error!("Image import task failed: {:?}", e),
                            }
                        });
                    },
                    "choose…"
                }
                span {
                    class: if missing { "image_name image_missing" } else { "image_name" },
                    title: current.clone().unwrap_or_default(),
                    if missing { "file not found: {shown_name}" } else { "{shown_name}" }
                }
                if current.is_some() {
                    button {
                        r#type: "button",
                        class: "image_button",
                        title: "remove image",
                        onclick: move |_| edit_image(&clear_props, |image| image.set(None)),
                        "×"
                    }
                }
            }
            if let Some(err) = import_error() {
                p { class: "image_missing", "{err}" }
            }
            if !recent.is_empty() {
                div { class: "image_row",
                    span { class: "image_recent_label", "recent" }
                    Dropdown {
                        options: recent_names,
                        selected: None,
                        onselect: move |index: usize| {
                            let Some(path) = recent.get(index).cloned() else { return };
                            edit_image(&recent_props, |image| image.set(Some(path)));
                        },
                    }
                }
            }
        }
    }
}
