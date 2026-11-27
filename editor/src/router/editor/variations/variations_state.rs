use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    is_rollable, own_roll_range, takes_roll_range, Project, RollMode, RollRange, RollRanges,
    Variations,
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

/// How the next roll goes, see [`RollMode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollKind {
    Uniform,
    Similar,
    Permutation,
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
    pub kind: Signal<RollKind>,
    /// How far a similar roll goes, in percent of each number's range.
    pub similar_percent: Signal<f32>,
    /// Values per number of a permutation roll.
    pub samples: Signal<usize>,
    /// Chance of a permutation to be rolled, in percent.
    pub keep_percent: Signal<f32>,
    /// Id of the roll that is running.
    pub rolling: SyncSignal<Option<u64>>,
    /// Scroll position of the grid, kept while the edit view is shown.
    pub scroll_top: Signal<f64>,
    /// The selected roll is shown next to the start, toggled with Y.
    pub compare: Signal<bool>,
}

pub fn use_variations_state() -> VariationsState {
    VariationsState {
        view: use_signal(|| EditorView::Edit),
        variations: use_signal_sync(|| None),
        selected: use_signal(BTreeSet::new),
        ranges: use_signal(BTreeMap::new),
        count: use_signal(|| 100),
        kind: use_signal(|| RollKind::Uniform),
        similar_percent: use_signal(|| 20.0),
        samples: use_signal(|| 3),
        keep_percent: use_signal(|| 100.0),
        rolling: use_signal_sync(|| None),
        scroll_top: use_signal(|| 0.0),
        compare: use_signal(|| false),
    }
}

impl VariationsState {
    pub fn roll_mode(&self) -> RollMode {
        match (self.kind)() {
            RollKind::Uniform => RollMode::Uniform,
            RollKind::Similar => RollMode::Similar {
                percent: (self.similar_percent)(),
            },
            RollKind::Permutation => RollMode::Permutation {
                samples: (self.samples)(),
                keep: (self.keep_percent)(),
            },
        }
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
            let count = variations.rolls.len().saturating_sub(1).max(1);
            match variations.meta.mode {
                RollMode::Uniform => {
                    state.kind.set(RollKind::Uniform);
                    state.count.set(count);
                }
                RollMode::Similar { percent } => {
                    state.kind.set(RollKind::Similar);
                    state.similar_percent.set(percent);
                    state.count.set(count);
                }
                // the count of a permutation roll follows from the grid
                RollMode::Permutation { samples, keep } => {
                    state.kind.set(RollKind::Permutation);
                    state.samples.set(samples);
                    state.keep_percent.set(keep);
                }
            }
        }
    }
    state.variations.set(loaded);
}
