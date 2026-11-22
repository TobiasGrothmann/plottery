use std::sync::Arc;

use dioxus::prelude::*;
use plottery_lib::BundledFont;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    font_preview::{cached_preview, font_label, preview, FontPreview},
    params_editor::LeafProps,
    widgets::search_picker::{PickerRow, SearchPicker},
};

#[component]
fn FontPreviewSvg(preview: Arc<FontPreview>, height: f32) -> Element {
    let width = height * preview.width / preview.height;
    rsx! {
        svg {
            class: "font-preview",
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {preview.width} {preview.height}",
            preserve_aspect_ratio: "xMinYMid meet",
            path { d: "{preview.d}" }
        }
    }
}

#[component]
pub fn FontField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::Font(name) = &props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let selected = BundledFont::from_name(name);
    let selected_name = name.clone();

    // Rendering all previews takes ~0.1 s in release and ~0.7 s in debug, so warm the cache off
    // the UI thread; the rows re-render as they arrive.
    let mut warmed = use_signal(|| 0usize);
    use_hook(move || {
        spawn(async move {
            for font in BundledFont::ALL.iter().copied() {
                if cached_preview(font).is_none() {
                    let _ = tokio::task::spawn_blocking(move || preview(font)).await;
                    *warmed.write() += 1;
                }
            }
        });
    });
    let _ = warmed();

    let rows: Vec<PickerRow> = BundledFont::ALL
        .iter()
        .copied()
        .map(|font| PickerRow {
            key: font.name().to_string(),
            content: rsx! {
                if let Some(preview) = cached_preview(font) {
                    FontPreviewSvg { preview, height: 22.0 }
                } else {
                    div { class: "font-pending" }
                }
                span { class: "font-name", "{font_label(font)}" }
            },
        })
        .collect();

    let pick = move |name: String| {
        if BundledFont::from_name(&name) != selected {
            leaf.edit(|value| *value = ProjectParamValue::Font(name));
        }
    };

    rsx! {
        SearchPicker {
            rows,
            selected: selected.map(|font| font.name().to_string()),
            placeholder: "Search fonts",
            empty_text: "No font matches",
            title: "{selected_name}",
            onpick: pick,
            if let Some(font) = selected {
                FontPreviewSvg { preview: preview(font), height: 16.0 }
            } else {
                span { class: "hint", "unknown font '{selected_name}'" }
            }
        }
    }
}
