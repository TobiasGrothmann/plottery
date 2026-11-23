use dioxus::prelude::*;
use plottery_project::{can_roll_paths, is_rollable_number, MAX_ROLLS};

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{Button, Segmented, Variant},
    },
    router::editor::{
        editor_ctx::EditorCtx, running_state::RunningState,
        variations::variations_state::roll_ranges,
    },
};

/// The params to roll, how many times, and the button that does it.
#[component]
pub fn RollPanel() -> Element {
    let ctx = use_context::<EditorCtx>();
    let state = ctx.variations;
    let mut count = state.count;
    let mut selected = state.selected;
    let mut similar = state.similar;
    let mut percent = state.similar_percent;
    // the fields' text while typing, the values follow when it is a number
    let mut count_text = use_signal(|| count.peek().to_string());
    use_effect(move || count_text.set(count().to_string()));
    let mut percent_text = use_signal(|| percent.peek().to_string());
    use_effect(move || percent_text.set(percent().to_string()));

    // in the order of the params, without ticks the project lost
    let params = ctx.project_params.read().clone();
    let to_roll: Vec<String> = can_roll_paths(&params.list)
        .into_iter()
        .filter(|path| selected.read().contains(path))
        .collect();
    // numbers without a #[range] wait for both ends of theirs
    let (ranges, missing) = roll_ranges(&params, &to_roll, &state.ranges.read());
    let mode = state.roll_mode();
    // what a similar roll keeps as it is: all but numbers
    let mut probe = params.clone();
    let kept: Vec<String> = match similar() {
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
    let nothing_moves = !to_roll.is_empty() && kept.len() == to_roll.len();
    let ready = !to_roll.is_empty() && missing.is_empty() && !nothing_moves;
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
                            class: if missing.contains(&path) { "roll-param-chip missing" } else if kept.contains(&path) { "roll-param-chip kept" } else { "roll-param-chip" },
                            disabled: rolling,
                            title: if kept.contains(&path) { "Don't roll {path}. A similar roll keeps it as it is" } else { "Don't roll {path}" },
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
            div { class: "roll-panel-row",
                Segmented {
                    options: vec![(false, "Uniform".to_string()), (true, "Similar".to_string())],
                    value: similar(),
                    disabled: rolling,
                    onchange: move |on| similar.set(on),
                }
                if similar() {
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
