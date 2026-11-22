use dioxus::prelude::*;
use plottery_project::{rollable_paths, MAX_ROLLS};

use crate::{
    components::{
        icon::IconName,
        ui::{Button, Variant},
    },
    router::editor::{
        editor_ctx::EditorCtx, params_editor::widgets::number_input::NumberInput,
        running_state::RunningState,
    },
};

/// How many times to roll the ticked params, and the button that does it.
#[component]
pub fn RollPanel() -> Element {
    let ctx = use_context::<EditorCtx>();
    let state = ctx.variations;
    let mut count = state.count;
    // in the order of the params, without ticks the project lost
    let to_roll: Vec<String> = rollable_paths(&ctx.project_params.read().list)
        .into_iter()
        .filter(|path| state.selected.read().contains(path))
        .collect();
    let rolling = (state.rolling)().is_some();
    let progress = match (ctx.running_state)() {
        RunningState::Rolling { done, total } => format!("{done}/{total}"),
        _ => String::new(),
    };
    let label = match to_roll.is_empty() {
        true => "Tick the params to roll".to_string(),
        false => to_roll.join(", "),
    };
    let paths = to_roll.clone();
    let roll = move |_| {
        ctx.project_runner.peek().trigger_roll(
            ctx.release,
            paths.clone(),
            count(),
            ctx.running_state,
            ctx.console,
            state.rolling,
        );
    };

    rsx! {
        style { {include_str!("./roll_panel.css")} }
        div { class: "roll-panel",
            span { class: "roll-panel-label", title: "{label}", "{label}" }
            div { class: "roll-panel-row",
                div { class: "roll-panel-count",
                    NumberInput {
                        value: count() as f32,
                        integer: true,
                        affix: "×",
                        aria_label: "Number of rolls",
                        oncommit: move |value: f32| count.set((value.max(1.0) as usize).min(MAX_ROLLS)),
                    }
                }
                if rolling {
                    Button {
                        variant: Variant::Secondary,
                        onclick: move |_| ctx.project_runner.peek().stop(),
                        "Stop"
                    }
                    span { class: "roll-panel-progress", "{progress}" }
                } else {
                    Button {
                        variant: Variant::Primary,
                        icon: IconName::Dice,
                        disabled: to_roll.is_empty(),
                        onclick: roll,
                        "Roll"
                    }
                }
            }
        }
    }
}
