use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    is_rollable, own_roll_range, takes_roll_range, Project, RollRange, RollRanges, Variations,
};
use plottery_runtime::{
    project_param_value::ProjectParamValue, project_params_list_wrapper::ProjectParamsListWrapper,
};

use crate::router::editor::console_messages::ConsoleMessages;

/// What the open project shows: its drawing to edit, or its variations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorView {
    Edit,
    Variations,
}

/// The ends set for the range a number is rolled in. One left alone is that of its own `#[range]`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RangeInput {
    pub from: Option<f32>,
    pub to: Option<f32>,
}

impl RangeInput {
    /// The range to roll `value` in, `None` while an end is missing.
    pub fn resolve(self, value: &ProjectParamValue) -> Option<RollRange> {
        let own = own_roll_range(value);
        Some(RollRange {
            from: self.from.or(own.map(|own| own.from))?,
            to: self.to.or(own.map(|own| own.to))?,
        })
    }
}

/// The ranges to roll `paths` with, and the paths that can't roll before their range has both ends.
pub fn roll_ranges(
    params: &ProjectParamsListWrapper,
    paths: &[String],
    inputs: &BTreeMap<String, RangeInput>,
) -> (RollRanges, Vec<String>) {
    let mut probe = params.clone();
    let mut ranges = RollRanges::new();
    let mut missing = Vec::new();
    for path in paths {
        let Ok(value) = probe.value_at_path_mut(path) else {
            continue;
        };
        if !takes_roll_range(value) {
            continue;
        }
        let input = inputs.get(path).copied().unwrap_or_default();
        let untouched = input == RangeInput::default();
        match input.resolve(value) {
            Some(_) if untouched => {}
            Some(range) => {
                ranges.insert(path.clone(), range);
            }
            // an Option without a range still rolls whether it is on
            None if untouched && is_rollable(value) => {}
            None => missing.push(path.clone()),
        }
    }
    (ranges, missing)
}

/// Which view is shown, the stored variations of the open project and what to roll next.
#[derive(Clone, Copy)]
pub struct VariationsState {
    pub view: Signal<EditorView>,
    /// Read while the variations view is open, `None` when there are none.
    pub variations: SyncSignal<Option<Variations>>,
    /// Dotted paths of the params to roll.
    pub selected: Signal<BTreeSet<String>>,
    /// The ranges set for numbers, by dotted path.
    pub ranges: Signal<BTreeMap<String, RangeInput>>,
    pub count: Signal<usize>,
    /// Id of the roll that is running.
    pub rolling: SyncSignal<Option<u64>>,
    /// Scroll position of the grid, kept while the edit view is shown.
    pub scroll_top: Signal<f64>,
}

pub fn use_variations_state() -> VariationsState {
    VariationsState {
        view: use_signal(|| EditorView::Edit),
        variations: use_signal_sync(|| None),
        selected: use_signal(BTreeSet::new),
        ranges: use_signal(BTreeMap::new),
        count: use_signal(|| 24),
        rolling: use_signal_sync(|| None),
        scroll_top: use_signal(|| 0.0),
    }
}

/// Reads the stored variations again, off the UI thread. Nothing selected yet selects what they
/// rolled, with their ranges.
pub async fn reload_variations(
    project: Project,
    mut state: VariationsState,
    console: SyncSignal<ConsoleMessages>,
) {
    let loaded = match tokio::task::spawn_blocking(move || project.load_variations()).await {
        Ok(Ok(loaded)) => loaded,
        Ok(Err(e)) => {
            return console
                .peek()
                .error(&format!("Failed to read the variations: {e:#}"))
        }
        Err(e) => return tracing::error!("Reading the variations panicked: {e}"),
    };
    if let Some(variations) = &loaded {
        if state.selected.peek().is_empty() && state.rolling.peek().is_none() {
            state
                .selected
                .set(variations.meta.params.iter().cloned().collect());
            let ranges = variations.meta.ranges.iter().map(|(path, range)| {
                let input = RangeInput {
                    from: Some(range.from),
                    to: Some(range.to),
                };
                (path.clone(), input)
            });
            state.ranges.set(ranges.collect());
            state
                .count
                .set(variations.rolls.len().saturating_sub(1).max(1));
        }
    }
    state.variations.set(loaded);
}
