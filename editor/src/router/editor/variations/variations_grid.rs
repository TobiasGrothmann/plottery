use dioxus::prelude::*;
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;

use super::{
    roll_panel::RollPanel, roll_tile::RollTile, variations_state::EditorView,
    variations_watch::use_variations_watch,
};
use crate::{
    components::tile_size_slider::TileSizeSlider,
    model::app_state::{update_app_state, AppState},
    router::editor::editor_ctx::EditorCtx,
};

/// The rolls of the open project as tiles, updated as they finish.
#[component]
pub fn VariationsGrid() -> Element {
    let ctx = use_context::<EditorCtx>();
    use_variations_watch(ctx);
    let app_state = use_context::<Signal<AppState>>();
    let tile_size = app_state.read().roll_tile_size;
    let project = ctx.project.read().clone();
    let variations = ctx.variations.variations.read().clone();

    let (tiles, status) = match variations {
        Some(variations) if !variations.rolls.is_empty() => {
            let done = variations.rolls.iter().filter(|roll| roll.done).count();
            let status = format!(
                "{done} of {} done · rolled {}",
                variations.rolls.len(),
                variations.meta.params.join(", ")
            );
            let set_id = variations.meta.created.timestamp_millis() as u32;
            let tiles = variations
                .rolls
                .into_iter()
                .map(|roll| {
                    let png = roll.done.then(|| {
                        project
                            .get_roll_path(roll.index, "png")
                            .to_string_lossy()
                            .to_string()
                    });
                    rsx! {
                        RollTile {
                            key: "{roll.index}",
                            index: roll.index,
                            png,
                            values: roll.values,
                            set_id,
                            on_pick: move |index| pick_roll(ctx, index),
                        }
                    }
                })
                .collect::<Vec<_>>();
            (Some(tiles), status)
        }
        _ => (None, "No variations yet".to_string()),
    };

    rsx! {
        style { {include_str!("./variations.css")} }
        div { class: "variations",
            RollPanel {}
            div { class: "variations-scroll", style: "--tile-size: {tile_size}px;",
                if let Some(tiles) = tiles {
                    div { class: "grid_cards", {tiles.into_iter()} }
                } else {
                    div { class: "variations-empty",
                        p { "No variations yet." }
                        p { class: "hint", "Click the params on the left to pick what to roll, then Roll." }
                    }
                }
            }
            div { class: "variations-bar",
                span { class: "variations-status", "{status}" }
                TileSizeSlider {
                    value: tile_size,
                    on_change: move |size| update_app_state(app_state, |state| state.set_roll_tile_size(size)),
                }
            }
        }
    }
}

/// Makes the params of roll `index` the project's and shows them in the edit view.
fn pick_roll(ctx: EditorCtx, index: usize) {
    let picked = ctx
        .variations
        .variations
        .peek()
        .as_ref()
        .and_then(|variations| {
            variations
                .rolls
                .iter()
                .find(|roll| roll.index == index)
                .map(|roll| roll.params.clone())
        });
    let Some(picked) = picked else {
        return;
    };
    let mut project_params = ctx.project_params;
    let defaults = project_params.peek().defaults.clone();
    // without defaults (no run yet) the run merges them
    let params = match defaults.is_empty() {
        true => picked,
        false => ProjectParamsListWrapper::new_combined(&picked.list, &defaults),
    };
    project_params.set(params);
    let mut view = ctx.variations.view;
    view.set(EditorView::Edit);
    ctx.trigger_run();
}
