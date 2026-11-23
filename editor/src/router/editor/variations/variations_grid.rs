use std::rc::Rc;

use dioxus::{html::geometry::PixelsVector2D, prelude::*};
use plottery_project::Roll;
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;

use super::{
    roll_nav::on_arrow_key,
    roll_panel::RollPanel,
    roll_tile::{CurrentTile, RollTile},
    variations_state::EditorView,
    variations_watch::use_variations_watch,
};
use crate::{
    components::tile_size_slider::TileSizeSlider,
    model::app_state::{update_app_state, AppState, ROLL_TILE_SIZE_MAX},
    router::editor::editor_ctx::EditorCtx,
};

/// The rolls of the open project as tiles, updated as they finish. `preview` is the url of the
/// current drawing.
#[component]
pub fn VariationsGrid(preview: Option<String>) -> Element {
    let ctx = use_context::<EditorCtx>();
    use_variations_watch(ctx);
    let mut scroll_top = ctx.variations.scroll_top;
    let mut scroller: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let mut restored = use_signal(|| false);
    // back from the edit view: scroll to where the grid was, once its tiles are there
    use_effect(move || {
        let has_tiles = ctx
            .variations
            .variations
            .read()
            .as_ref()
            .is_some_and(|variations| !variations.rolls.is_empty());
        let Some(element) = scroller() else {
            return;
        };
        if !has_tiles || *restored.peek() {
            return;
        }
        restored.set(true);
        let top = *scroll_top.peek();
        if top > 0.0 {
            spawn(async move {
                let _ = element
                    .scroll(PixelsVector2D::new(0.0, top), ScrollBehavior::Instant)
                    .await;
            });
        }
    });
    let app_state = use_context::<Signal<AppState>>();
    let tile_size = app_state.read().roll_tile_size;
    let project = ctx.project.read().clone();
    let variations = ctx.variations.variations.read().clone();
    let rolling = ctx.variations.rolling.read().is_some()
        || variations.as_ref().is_some_and(|v| v.progress().is_some());
    let current_params = ctx.project_params.read().list.clone();

    let set_id = variations.as_ref().map_or(0, |variations| {
        variations.meta.created.timestamp_millis() as u32
    });
    let rolls = variations
        .map(|variations| variations.rolls)
        .unwrap_or_default();
    let is_current = |roll: &Roll| !roll.skipped && roll.params.list == current_params;
    // the next roll starts from the current drawing, shown first when no roll is it
    let show_current = !rolls.iter().any(is_current);
    let tiles = rolls.iter().map(|roll| {
        let png = project
            .get_roll_path(roll.index, "png")
            .to_string_lossy()
            .to_string();
        rsx! {
            RollTile {
                key: "{roll.index}",
                index: roll.index,
                png,
                current: is_current(roll),
                skipped: roll.skipped,
                set_id,
                on_select: move |index| pick_roll(ctx, index, false),
                on_open: move |index| pick_roll(ctx, index, true),
            }
        }
    });
    let mut view = ctx.variations.view;

    rsx! {
        style { {include_str!("./variations.css")} }
        div { class: "variations",
            RollPanel {}
            div {
                class: "variations-scroll",
                style: "--tile-size: {tile_size}px;",
                onmounted: move |event| scroller.set(Some(event.data())),
                onscroll: move |event| scroll_top.set(event.data().scroll_top()),
                // no text fields in here, so the arrows always move the selection
                onkeydown: move |event| {
                    if on_arrow_key(ctx, &event.key()) {
                        event.prevent_default();
                        event.stop_propagation();
                    }
                },
                div { class: "grid_cards",
                    if show_current {
                        CurrentTile {
                            src: preview,
                            on_open: move |_| view.set(EditorView::Edit),
                        }
                    }
                    {tiles}
                }
                if rolls.is_empty() {
                    div { class: "variations-empty",
                        if rolling {
                            p { "Rolling…" }
                            p { class: "hint", "Each roll shows up when it is done." }
                        } else {
                            p { "No variations yet." }
                            p { class: "hint", "Click the params on the left to pick what to roll, then Roll." }
                        }
                    }
                }
            }
            div { class: "variations-bar",
                TileSizeSlider {
                    value: tile_size,
                    max: ROLL_TILE_SIZE_MAX,
                    on_change: move |size| update_app_state(app_state, |state| state.set_roll_tile_size(size)),
                }
            }
        }
    }
}

/// Makes the params and drawing of roll `index` the project's, the start of the next roll, and
/// with `open` shows them in the edit view. Nothing runs, so a roll in progress goes on.
pub(super) fn pick_roll(ctx: EditorCtx, index: usize, open: bool) {
    let picked = ctx
        .variations
        .variations
        .peek()
        .as_ref()
        .and_then(|variations| {
            variations
                .rolls
                .iter()
                .find(|roll| roll.index == index && !roll.skipped)
                .map(|roll| roll.params.clone())
        });
    let Some(picked) = picked else {
        return;
    };
    let mut view = ctx.variations.view;
    // the clicks of a double click pick it already
    if picked.list == ctx.project_params.peek().list {
        if open {
            view.set(EditorView::Edit);
        }
        return;
    }
    let layer = match ctx.project.peek().load_roll_layer(index) {
        Ok(layer) => layer,
        Err(e) => {
            return ctx
                .console
                .peek()
                .error(&format!("Failed to open roll {index}: {e:#}"))
        }
    };
    let mut project_params = ctx.project_params;
    let defaults = project_params.peek().defaults.clone();
    // without defaults (no run yet) the next run merges them
    let params = match defaults.is_empty() {
        true => picked,
        false => ProjectParamsListWrapper::new_combined(&picked.list, &defaults),
    };
    project_params.set(params);
    ctx.project_runner.peek().show_layer(layer, ctx.console);
    if open {
        view.set(EditorView::Edit);
    }
}
