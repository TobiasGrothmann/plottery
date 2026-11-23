use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use plottery_lib::Rect;
use plottery_project::Project;

use crate::{
    components::{icon::IconName, image::SVGImage, loading_spinner::Loading, ui::IconButton},
    router::editor::editor_ctx::EditorCtx,
    util::preview_zoom::PreviewZoom,
};

/// The start of the rolls next to the selected roll, fullscreen, each side zoomed on its own.
/// Y, Esc or the close button ends it.
#[component]
pub fn RollCompare() -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut compare = ctx.variations.compare;
    let current_params = ctx.project_params.read().list.clone();
    let variations = ctx.variations.variations.read().clone();
    let set_id = variations.as_ref().map_or(0, |variations| {
        variations.meta.created.timestamp_millis() as u32
    });
    let selected = variations.and_then(|variations| {
        variations
            .rolls
            .iter()
            .find(|roll| !roll.skipped && roll.index != 0 && roll.params.list == current_params)
            .map(|roll| roll.index)
    });

    rsx! {
        style { {include_str!("./roll_compare.css")} }
        section { class: "roll-compare",
            match selected {
                Some(index) => rsx! {
                    CompareSide { key: "{set_id}-0", index: 0 }
                    div { class: "roll-compare-wall" }
                    CompareSide { key: "{set_id}-{index}", index }
                },
                None => rsx! {
                    p { class: "roll-compare-hint", "Click a roll to compare it with the start." }
                },
            }
            div { class: "roll-compare-close",
                IconButton {
                    icon: IconName::X,
                    label: "Close compare",
                    title: "Close compare (Y, Esc)",
                    onclick: move |_| compare.set(false),
                }
            }
        }
    }
}

/// Roll `index` as a zoomable drawing, like the fullscreen preview.
#[component]
fn CompareSide(index: usize) -> Element {
    let ctx = use_context::<EditorCtx>();
    let zoom = use_signal(PreviewZoom::default);
    let svg = use_resource(move || {
        let project = ctx.project.peek().clone();
        async move {
            match tokio::task::spawn_blocking(move || roll_svg(&project, index)).await {
                Ok(Ok(svg)) => Ok(svg),
                Ok(Err(e)) => Err(format!("{e:#}")),
                Err(e) => Err(e.to_string()),
            }
        }
    });
    let name = match index {
        0 => "start".to_string(),
        _ => index.to_string(),
    };
    let side = match &*svg.read() {
        None => rsx! { Loading {} },
        Some(Err(e)) => rsx! { p { class: "roll-compare-hint", "{e}" } },
        Some(Ok((src, layer_bbox))) => rsx! {
            SVGImage {
                stage_id: format!("roll-compare-{index}"),
                src: src.clone(),
                layer_bbox: *layer_bbox,
                pick_active: false,
                onpick: move |_| {},
                mark: None,
                zoom,
                onready: move |_| {},
            }
        },
    };

    rsx! {
        div { class: "roll-compare-pane",
            {side}
            span { class: "roll_number", "{name}" }
        }
    }
}

/// Writes the svg of roll `index` next to the editor's preview in the cache, returns its url and
/// the bounding box of its layer.
fn roll_svg(project: &Project, index: usize) -> anyhow::Result<(String, Option<Rect>)> {
    let layer = project.load_roll_layer(index)?;
    let path = project
        .get_editor_preview_image_path()?
        .with_extension(format!("roll-{index}.svg"));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, layer.to_svg())?;
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| since_epoch.as_millis());
    Ok((format!("{}?{millis}", path.display()), layer.bounding_box()))
}
