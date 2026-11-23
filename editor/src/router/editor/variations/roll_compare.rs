use dioxus::prelude::*;

use crate::{
    components::{icon::IconName, image::Image, ui::IconButton},
    router::editor::editor_ctx::EditorCtx,
};

/// The start of the rolls next to the selected roll. Y or the close button ends it.
#[component]
pub fn RollCompare(fullscreen: Signal<bool>) -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut compare = ctx.variations.compare;
    let project = ctx.project.read().clone();
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
    let png = |index: usize| {
        project
            .get_roll_path(index, "png")
            .to_string_lossy()
            .to_string()
    };

    rsx! {
        style { {include_str!("./roll_compare.css")} }
        section {
            class: if fullscreen() { "roll-compare fullscreen" } else { "roll-compare" },
            div { class: "roll-compare-close",
                IconButton {
                    icon: IconName::X,
                    label: "Close compare",
                    title: "Close compare (Y)",
                    onclick: move |_| compare.set(false),
                }
            }
            match selected {
                Some(index) => rsx! {
                    div { class: "roll-compare-pane",
                        Image { img_path: png(0), redraw_counter: set_id, class: "roll-compare-paper" }
                        span { class: "roll_number", "start" }
                    }
                    div { class: "roll-compare-pane",
                        Image { img_path: png(index), redraw_counter: set_id, class: "roll-compare-paper" }
                        span { class: "roll_number", "{index}" }
                    }
                },
                None => rsx! {
                    p { class: "roll-compare-hint", "Click a roll to compare it with the start." }
                },
            }
        }
    }
}
