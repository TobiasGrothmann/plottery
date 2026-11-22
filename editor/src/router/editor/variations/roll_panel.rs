use dioxus::prelude::*;
use plottery_project::{rollable_paths, MAX_ROLLS};

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{Button, Variant},
    },
    router::editor::{editor_ctx::EditorCtx, running_state::RunningState},
};

/// The params to roll, how many times, and the button that does it.
#[component]
pub fn RollPanel() -> Element {
    let ctx = use_context::<EditorCtx>();
    let state = ctx.variations;
    let mut count = state.count;
    let mut selected = state.selected;
    // the field's text while typing, the count follows when it is a number
    let mut count_text = use_signal(|| count.peek().to_string());
    use_effect(move || count_text.set(count().to_string()));

    // in the order of the params, without ticks the project lost
    let to_roll: Vec<String> = rollable_paths(&ctx.project_params.read().list)
        .into_iter()
        .filter(|path| selected.read().contains(path))
        .collect();
    let rolling = (state.rolling)().is_some();
    let progress = match (ctx.running_state)() {
        RunningState::Rolling { done, total } => Some((done, total)),
        _ => None,
    };
    let paths = to_roll.clone();
    let roll = Callback::new(move |_: ()| {
        if paths.is_empty() || state.rolling.peek().is_some() {
            return;
        }
        ctx.project_runner.peek().trigger_roll(
            ctx.release,
            paths.clone(),
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
                            class: "roll-param-chip",
                            disabled: rolling,
                            title: "Don't roll {path}",
                            onclick: move |_| {
                                selected.write().remove(&path);
                            },
                            "{path}"
                            span { class: "roll-param-chip-x", "×" }
                        }
                    }
                }
            }
            div { class: "roll-panel-row",
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
                        disabled: to_roll.is_empty(),
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
