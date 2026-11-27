use dioxus::prelude::*;
use std::collections::BTreeSet;

use plottery_project::{
    can_roll_paths, is_rollable_number, permutation_info, RollMode, MAX_PERMUTATION_SAMPLES,
    MAX_ROLLS,
};
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{Button, Segmented, Variant},
    },
    router::editor::{
        editor_ctx::EditorCtx,
        running_state::RunningState,
        variations::variations_state::{roll_ranges, RollKind},
    },
};

/// The ticked params the project still has, in the order of its params.
fn ticked_paths(params: &ProjectParamsListWrapper, selected: &BTreeSet<String>) -> Vec<String> {
    can_roll_paths(&params.list)
        .into_iter()
        .filter(|path| selected.contains(path))
        .collect()
}

/// The params to roll, how many times, and the button that does it.
#[component]
pub fn RollPanel() -> Element {
    let ctx = use_context::<EditorCtx>();
    let state = ctx.variations;
    let mut count = state.count;
    let mut selected = state.selected;
    let mut kind = state.kind;
    let mut percent = state.similar_percent;
    let mut samples = state.samples;
    let mut keep = state.keep_percent;
    // the fields' text while typing, the values follow when it is a number
    let mut count_text = use_signal(|| count.peek().to_string());
    use_effect(move || count_text.set(count().to_string()));
    let mut percent_text = use_signal(|| percent.peek().to_string());
    use_effect(move || percent_text.set(percent().to_string()));
    let mut samples_text = use_signal(|| samples.peek().to_string());
    use_effect(move || samples_text.set(samples().to_string()));
    let mut keep_text = use_signal(|| keep.peek().to_string());
    use_effect(move || keep_text.set(keep().to_string()));
    // only again when what it depends on changes, not on every finished roll
    let permutations = use_memo(move || {
        let RollMode::Permutation { samples, keep } = state.roll_mode() else {
            return None;
        };
        let params = ctx.project_params.read();
        let to_roll = ticked_paths(&params, &state.selected.read());
        let (ranges, _) = roll_ranges(&params, &to_roll, &state.ranges.read());
        Some(permutation_info(&params, &to_roll, &ranges, samples, keep))
    });

    // in the order of the params, without ticks the project lost
    let params = ctx.project_params.read().clone();
    let to_roll = ticked_paths(&params, &selected.read());
    // numbers without a #[range] wait for both ends of theirs
    let (ranges, missing) = roll_ranges(&params, &to_roll, &state.ranges.read());
    let mode = state.roll_mode();
    // what a similar roll keeps as it is: all but numbers
    let mut probe = params.clone();
    let unmoved: Vec<String> = match kind() == RollKind::Similar {
        true => to_roll
            .iter()
            .filter(|path| {
                !probe
                    .value_at_path_mut(path)
                    .is_ok_and(|value| is_rollable_number(value))
            })
            .cloned()
            .collect(),
        false => Vec::new(),
    };
    let nothing_moves = !to_roll.is_empty() && unmoved.len() == to_roll.len();
    let no_permutation = !to_roll.is_empty()
        && missing.is_empty()
        && permutations().is_some_and(|permutations| permutations.rolls == 0);
    let ready = !to_roll.is_empty() && missing.is_empty() && !nothing_moves && !no_permutation;
    let missing_names = missing.join(", ");
    let rolling = (state.rolling)().is_some();
    let progress = match (ctx.running_state)() {
        RunningState::Rolling { done, total } => Some((done, total)),
        _ => None,
    };
    // a roll from the terminal, seen in the stored files
    let elsewhere = match rolling {
        true => None,
        false => state.variations.read().as_ref().and_then(|v| v.progress()),
    };
    let paths = to_roll.clone();
    let roll = Callback::new(move |_: ()| {
        if !ready || state.rolling.peek().is_some() {
            return;
        }
        ctx.project_runner.peek().trigger_roll(
            ctx.release,
            paths.clone(),
            ranges.clone(),
            mode,
            *count.peek(),
            ctx.running_state,
            ctx.console,
            state.rolling,
        );
    });

    rsx! {
        style { {include_str!("./roll_panel.css")} }
        div { class: "roll-panel",
            div { class: "roll-panel-head",
                Icon { name: IconName::Dice }
                span { class: "roll-panel-title", "Roll" }
            }
            if to_roll.is_empty() {
                span { class: "roll-panel-empty", "Click the params on the left to roll them" }
            } else {
                div { class: "roll-panel-chips",
                    for path in to_roll.iter().cloned() {
                        button {
                            key: "{path}",
                            r#type: "button",
                            class: if missing.contains(&path) { "roll-param-chip missing" } else if unmoved.contains(&path) { "roll-param-chip kept" } else { "roll-param-chip" },
                            disabled: rolling,
                            title: if unmoved.contains(&path) { "Don't roll {path}. A similar roll keeps it as it is" } else { "Don't roll {path}" },
                            onclick: move |_| {
                                selected.write().remove(&path);
                            },
                            "{path}"
                            span { class: "roll-param-chip-x", "×" }
                        }
                    }
                }
            }
            if !missing.is_empty() {
                span { class: "roll-panel-missing", "Set the range to roll {missing_names} in" }
            }
            if nothing_moves {
                span { class: "roll-panel-missing", "A similar roll only moves numbers, tick one" }
            }
            if no_permutation {
                span { class: "roll-panel-missing", "No permutation to roll, use more samples, params or a higher keep" }
            }
            div { class: "roll-panel-row",
                Segmented {
                    options: vec![
                        (RollKind::Uniform, "Uniform".to_string()),
                        (RollKind::Similar, "Similar".to_string()),
                        (RollKind::Permutation, "Permutation".to_string()),
                    ],
                    value: kind(),
                    disabled: rolling,
                    onchange: move |new| kind.set(new),
                }
                if kind() == RollKind::Similar {
                    label {
                        class: "roll-count",
                        title: "How far from the current values, in percent of each number's range",
                        span { "±" }
                        input {
                            class: "input",
                            inputmode: "decimal",
                            "aria-label": "Percent of the range to roll near the current values",
                            disabled: rolling,
                            value: "{percent_text}",
                            oninput: move |event| {
                                let text = event.value();
                                if let Ok(value) = text.trim().parse::<f32>() {
                                    if value.is_finite() {
                                        percent.set(value.clamp(1.0, 100.0));
                                    }
                                }
                                percent_text.set(text);
                            },
                            onblur: move |_| percent_text.set(percent().to_string()),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter {
                                    percent_text.set(percent().to_string());
                                    roll.call(());
                                }
                            },
                        }
                        span { "%" }
                    }
                }
                if kind() == RollKind::Permutation {
                    div { class: "roll-permutation",
                        div { class: "roll-permutation-fields",
                            label {
                                class: "roll-count",
                                title: "Values per number from one end of its range to the other. Bools, enums and options take all their choices",
                                input {
                                    class: "input",
                                    inputmode: "numeric",
                                    "aria-label": "Samples per param",
                                    disabled: rolling,
                                    value: "{samples_text}",
                                    oninput: move |event| {
                                        let text = event.value();
                                        if let Ok(value) = text.trim().parse::<usize>() {
                                            samples.set(value.clamp(2, MAX_PERMUTATION_SAMPLES));
                                        }
                                        samples_text.set(text);
                                    },
                                    onblur: move |_| samples_text.set(samples().to_string()),
                                    onkeydown: move |event| {
                                        if event.key() == Key::Enter {
                                            samples_text.set(samples().to_string());
                                            roll.call(());
                                        }
                                    },
                                }
                                span { "samples" }
                            }
                            label {
                                class: "roll-count",
                                title: "The chance of each permutation to be rolled, the same ones every time",
                                span { "keep" }
                                input {
                                    class: "input",
                                    inputmode: "decimal",
                                    "aria-label": "Percent of the permutations to roll",
                                    disabled: rolling,
                                    value: "{keep_text}",
                                    oninput: move |event| {
                                        let text = event.value();
                                        if let Ok(value) = text.trim().parse::<f32>() {
                                            if value.is_finite() {
                                                keep.set(value.clamp(0.01, 100.0));
                                            }
                                        }
                                        keep_text.set(text);
                                    },
                                    onblur: move |_| keep_text.set(keep().to_string()),
                                    onkeydown: move |event| {
                                        if event.key() == Key::Enter {
                                            keep_text.set(keep().to_string());
                                            roll.call(());
                                        }
                                    },
                                }
                                span { "%" }
                            }
                        }
                        if let Some(permutations) = permutations().filter(|_| !to_roll.is_empty()) {
                            div { class: "roll-panel-info", {permutations.summary()} }
                        }
                    }
                } else {
                    label { class: "roll-count",
                        input {
                            class: "input",
                            inputmode: "numeric",
                            "aria-label": "Number of rolls",
                            disabled: rolling,
                            value: "{count_text}",
                            oninput: move |event| {
                                let text = event.value();
                                if let Ok(value) = text.trim().parse::<usize>() {
                                    count.set(value.clamp(1, MAX_ROLLS));
                                }
                                count_text.set(text);
                            },
                            onblur: move |_| count_text.set(count().to_string()),
                            onkeydown: move |event| {
                                if event.key() == Key::Enter {
                                    count_text.set(count().to_string());
                                    roll.call(());
                                }
                            },
                        }
                        span { "times" }
                    }
                }
                if rolling {
                    Button {
                        variant: Variant::Secondary,
                        onclick: move |_| ctx.project_runner.peek().stop(),
                        "Stop"
                    }
                } else {
                    Button {
                        variant: Variant::Primary,
                        icon: IconName::Dice,
                        disabled: !ready,
                        onclick: move |_| roll.call(()),
                        "Roll"
                    }
                }
            }
            if rolling {
                div { class: "roll-progress",
                    ProgressBar { progress: progress.map(|(done, total)| done as f32 / total.max(1) as f32) }
                    match progress {
                        Some((done, total)) => rsx! { span { "{done} / {total}" } },
                        None => rsx! { span { "Compiling…" } },
                    }
                }
            } else if let Some((done, total)) = elsewhere {
                div { class: "roll-progress", title: "Rolled outside the editor, e.g. with plottery roll",
                    ProgressBar { progress: Some(done as f32 / total.max(1) as f32) }
                    span { "{done} / {total} in the terminal" }
                }
            }
        }
    }
}

/// A thin bar filled to `progress` (0 to 1), moving back and forth while unknown.
#[component]
pub fn ProgressBar(progress: Option<f32>) -> Element {
    match progress {
        Some(progress) => {
            let percent = (progress.clamp(0.0, 1.0) * 100.0).round();
            rsx! {
                div { class: "progress-bar", role: "progressbar", "aria-valuenow": "{percent}",
                    div { style: "width: {percent}%;" }
                }
            }
        }
        None => rsx! {
            div { class: "progress-bar indeterminate", role: "progressbar",
                div {}
            }
        },
    }
}
