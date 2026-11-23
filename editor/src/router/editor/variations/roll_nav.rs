use dioxus::prelude::*;

use super::variations_grid::pick_roll;
use crate::router::editor::editor_ctx::EditorCtx;

/// Columns of the variations grid, as laid out on screen.
const GRID_COLUMNS: &str = r#"
const grid = document.querySelector(".variations .grid_cards");
return grid ? getComputedStyle(grid).gridTemplateColumns.split(" ").length : 1;
"#;

/// The roll `step` tiles away from tile `from`, going on past tiles that can't be picked.
/// `tiles` holds the roll index of each tile in grid order, `None` for one that can't be picked.
pub fn step_roll(tiles: &[Option<usize>], from: usize, step: isize) -> Option<usize> {
    if step == 0 {
        return None;
    }
    let mut at = from as isize + step;
    while at >= 0 && (at as usize) < tiles.len() {
        if let Some(index) = tiles[at as usize] {
            return Some(index);
        }
        at += step.signum();
    }
    None
}

/// Picks the roll an arrow key points to, rows apart for up and down. Returns whether `key` is
/// an arrow.
pub fn on_arrow_key(ctx: EditorCtx, key: &Key) -> bool {
    let rows = match key {
        Key::ArrowLeft => Some(-1),
        Key::ArrowRight => Some(1),
        _ => None,
    };
    let columns = match key {
        Key::ArrowUp => Some(-1),
        Key::ArrowDown => Some(1),
        _ => None,
    };
    match (rows, columns) {
        (Some(step), _) => step_selection(ctx, step),
        (_, Some(direction)) => {
            spawn(async move {
                let columns = document::eval(GRID_COLUMNS)
                    .await
                    .ok()
                    .and_then(|columns| columns.as_i64())
                    .unwrap_or(1)
                    .max(1);
                step_selection(ctx, direction * columns as isize);
            });
        }
        _ => return false,
    }
    true
}

/// Picks the roll `step` tiles from the selected one and brings its tile into view.
fn step_selection(ctx: EditorCtx, step: isize) {
    let compare = *ctx.variations.compare.peek();
    let current_params = ctx.project_params.peek().list.clone();
    let target = {
        let variations = ctx.variations.variations.peek();
        let Some(variations) = variations.as_ref() else {
            return;
        };
        let is_current =
            |roll: &plottery_project::Roll| !roll.skipped && roll.params.list == current_params;
        // the grid shows the current drawing first when no roll is it
        let current_tile = (!variations.rolls.iter().any(is_current)).then_some(None);
        // the compare shows the start on its own side
        let rolls = variations.rolls.iter().map(|roll| {
            let start_beside = compare && roll.index == 0;
            (!roll.skipped && !start_beside).then_some(roll.index)
        });
        let tiles: Vec<Option<usize>> = current_tile.into_iter().chain(rolls).collect();
        let from = variations
            .rolls
            .iter()
            .position(is_current)
            .map_or(0, |at| at + usize::from(current_tile.is_some()));
        step_roll(&tiles, from, step)
    };
    let Some(index) = target else {
        return;
    };
    pick_roll(ctx, index, false);
    document::eval(&format!(
        r#"const tile = document.getElementById("roll-tile-{index}");
if (tile) {{
  tile.focus({{ preventScroll: true }});
  tile.scrollIntoView({{ block: "nearest" }});
}}"#
    ));
}
