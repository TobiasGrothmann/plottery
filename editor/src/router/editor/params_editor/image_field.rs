use std::{
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{file_name_of, project_param_value::ProjectParamValue, ProjectParamImage};
use rfd::FileDialog;

use crate::{
    components::{
        icon::IconName,
        ui::{Button, Checkbox, IconButton, Size},
    },
    router::editor::params_editor::{
        params_editor::LeafProps,
        widgets::{dropdown::Dropdown, segmented::Segmented},
    },
    util::image_import::{import_image, resolve_image_path, DOWNSCALE_MAX_PX},
};

/// Cache-busting key so a re-imported file with the same name refreshes the preview.
fn file_mtime_millis(path: &Path) -> u128 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

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
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let mut copy_mode = use_signal(|| true);
    let mut downscale = use_signal(|| true);
    let mut import_error = use_signal_sync(|| None::<String>);

    let current = image.path.clone();
    let resolved = current
        .as_deref()
        .map(|stored| resolve_image_path(&props.project_dir, stored));
    let missing = resolved.as_ref().map(|p| !p.exists()).unwrap_or(false);
    let preview_src = resolved
        .filter(|p| p.exists())
        .map(|p| format!("{}?{}", p.to_string_lossy(), file_mtime_millis(&p)));
    let shown_name = current
        .as_deref()
        .map(file_name_of)
        .unwrap_or_else(|| "No image selected".to_string());
    let recent = image.recent.clone();
    let recent_names: Vec<String> = recent.iter().map(|p| file_name_of(p)).collect();

    let choose_props = props.clone();
    let recent_props = props.clone();
    let clear_props = props.clone();
    let project_dir: PathBuf = props.project_dir.clone();

    rsx! {
        style { {include_str!("image_field.css")} }
        div {
            Segmented {
                options: vec!["Copy into project".to_string(), "Link to original".to_string()],
                selected: Some(if copy_mode() { 0 } else { 1 }),
                onselect: move |index: usize| copy_mode.set(index == 0),
            }
        }
        if copy_mode() {
            Checkbox {
                class: "image-downscale",
                checked: downscale(),
                label: "Downscale copy to {DOWNSCALE_MAX_PX} px max",
                onchange: move |checked: bool| downscale.set(checked),
            }
        }
        div { class: "hrow",
            Button {
                size: Size::Sm,
                icon: IconName::Image,
                onclick: move |_| {
                    let Some(picked) = FileDialog::new()
                        .add_filter("images", &["png", "jpg", "jpeg"])
                        .pick_file()
                        else {
                        return;
                    };
                    import_error.set(None);
                    if !copy_mode() {
                        edit_image(
                            &choose_props,
                            |image| { image.set(Some(picked.to_string_lossy().to_string())) },
                        );
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
                "Choose…"
            }
            span {
                class: if missing { "hint image-name image-missing" } else { "hint image-name" },
                title: current.clone().unwrap_or_default(),
                if missing {
                    "File not found: {shown_name}"
                } else {
                    "{shown_name}"
                }
            }
            if current.is_some() {
                IconButton {
                    icon: IconName::X,
                    label: "Remove image",
                    size: Size::Sm,
                    onclick: move |_| edit_image(&clear_props, |image| image.set(None)),
                }
            }
        }
        if let Some(err) = import_error() {
            span { class: "hint image-missing", "{err}" }
        }
        if let Some(src) = preview_src {
            img {
                class: "image-preview",
                src: "{src}",
                title: current.clone().unwrap_or_default(),
            }
        }
        if !recent.is_empty() {
            Dropdown {
                options: recent_names,
                selected: None,
                placeholder: "Recent images…",
                onselect: move |index: usize| {
                    let Some(path) = recent.get(index).cloned() else { return };
                    edit_image(&recent_props, |image| image.set(Some(path)));
                },
            }
        }
    }
}
