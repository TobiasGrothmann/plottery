use std::{
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::Result;
use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{file_name_of, project_param_value::ProjectParamValue, ProjectParamFile};
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
    util::file_import::{import_image, import_svg, resolve_stored_path, DOWNSCALE_MAX_PX},
};

/// Which file param a [`FileField`] edits.
#[derive(Clone, Copy, PartialEq)]
enum FileKind {
    Image,
    Svg,
}

impl FileKind {
    fn of(value: &ProjectParamValue) -> Option<(Self, &ProjectParamFile)> {
        match value {
            ProjectParamValue::Image(file) => Some((Self::Image, file)),
            ProjectParamValue::Svg(file) => Some((Self::Svg, file)),
            _ => None,
        }
    }

    fn file_mut(self, value: &mut ProjectParamValue) -> Option<&mut ProjectParamFile> {
        match (self, value) {
            (Self::Image, ProjectParamValue::Image(file)) => Some(file),
            (Self::Svg, ProjectParamValue::Svg(file)) => Some(file),
            _ => None,
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Self::Image => "image",
            Self::Svg => "svg",
        }
    }

    fn extensions(self) -> &'static [&'static str] {
        match self {
            Self::Image => &["png", "jpg", "jpeg"],
            Self::Svg => &["svg"],
        }
    }

    fn can_downscale(self) -> bool {
        self == Self::Image
    }

    fn import(self, project_dir: &Path, source: &Path, downscale: bool) -> Result<String> {
        match self {
            Self::Image => import_image(project_dir, source, downscale),
            Self::Svg => import_svg(project_dir, source),
        }
    }
}

/// Cache-busting key so a re-imported file with the same name refreshes the preview.
fn file_mtime_millis(path: &Path) -> u128 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn edit_file(props: &LeafProps, kind: FileKind, edit: impl FnOnce(&mut ProjectParamFile)) {
    props.edit(|value| {
        if let Some(file) = kind.file_mut(value) {
            edit(file);
        }
    });
}

/// Picker for `ImagePath` and `SvgPath` params: copy or link a file, preview it, pick recents.
#[component]
pub fn FileField(props: LeafProps) -> Element {
    let Some((kind, file)) = FileKind::of(&props.param.value) else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let mut copy_mode = use_signal(|| true);
    let mut downscale = use_signal(|| true);
    let mut import_error = use_signal_sync(|| None::<String>);

    let noun = kind.noun();
    let current = file.path.clone();
    let resolved = current
        .as_deref()
        .map(|stored| resolve_stored_path(&props.project_dir, stored));
    let missing = resolved.as_ref().map(|p| !p.exists()).unwrap_or(false);
    let preview_src = resolved
        .filter(|p| p.exists())
        .map(|p| format!("{}?{}", p.to_string_lossy(), file_mtime_millis(&p)));
    let shown_name = current
        .as_deref()
        .map(file_name_of)
        .unwrap_or_else(|| format!("No {noun} selected"));
    let recent = file.recent.clone();
    let recent_names: Vec<String> = recent.iter().map(|p| file_name_of(p)).collect();

    let choose_props = props.clone();
    let recent_props = props.clone();
    let clear_props = props.clone();
    let project_dir: PathBuf = props.project_dir.clone();

    rsx! {
        style { {include_str!("file_field.css")} }
        div {
            Segmented {
                options: vec!["Copy into project".to_string(), "Link to original".to_string()],
                selected: Some(if copy_mode() { 0 } else { 1 }),
                onselect: move |index: usize| copy_mode.set(index == 0),
            }
        }
        if copy_mode() && kind.can_downscale() {
            Checkbox {
                class: "file-downscale",
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
                        .add_filter(format!("{noun}s"), kind.extensions())
                        .pick_file()
                        else {
                        return;
                    };
                    import_error.set(None);
                    if !copy_mode() {
                        edit_file(
                            &choose_props,
                            kind,
                            |file| { file.set(Some(picked.to_string_lossy().to_string())) },
                        );
                        return;
                    }
                    let project_dir = project_dir.clone();
                    let props = choose_props.clone();
                    let downscale = downscale();
                    tokio::spawn(async move {
                        let result = tokio::task::spawn_blocking(move || {
                                kind.import(&project_dir, &picked, downscale)
                            })
                            .await;
                        match result {
                            Ok(Ok(stored)) => edit_file(&props, kind, |file| file.set(Some(stored))),
                            Ok(Err(e)) => {
                                tracing::error!("Failed to import {}: {:?}", noun, e);
                                if let Ok(mut err) = import_error.try_write() {
                                    *err = Some(format!("{e:#}"));
                                }
                            }
                            Err(e) => tracing::error!("File import task failed: {:?}", e),
                        }
                    });
                },
                "Choose…"
            }
            span {
                class: if missing { "hint file-name file-missing" } else { "hint file-name" },
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
                    label: "Remove {noun}",
                    size: Size::Sm,
                    onclick: move |_| edit_file(&clear_props, kind, |file| file.set(None)),
                }
            }
        }
        if let Some(err) = import_error() {
            span { class: "hint file-missing", "{err}" }
        }
        if let Some(src) = preview_src {
            img {
                class: if kind == FileKind::Svg { "file-preview file-preview-svg" } else { "file-preview" },
                src: "{src}",
                title: current.clone().unwrap_or_default(),
            }
        }
        if !recent.is_empty() {
            Dropdown {
                options: recent_names,
                selected: None,
                placeholder: "Recent {noun}s…",
                onselect: move |index: usize| {
                    let Some(path) = recent.get(index).cloned() else { return };
                    edit_file(&recent_props, kind, |file| file.set(Some(path)));
                },
            }
        }
    }
}
