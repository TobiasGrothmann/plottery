use dioxus::prelude::*;
use plottery_project::RollRange;

use crate::router::editor::{
    console_messages::ActiveConsole, editor_ctx::EditorCtx, params_editor::widgets::format_number,
    variations::variations_state::RangeInput,
};

/// From and to of the range a number is rolled in, in the variations view. They start as the
/// param's own `#[range]` (`own`); a number without one has to get both.
#[component]
pub fn RollRangeField(
    /// Dotted path of the param.
    dot: String,
    integer: bool,
    own: Option<RollRange>,
    /// The param is ticked and can't roll without both ends.
    required: bool,
) -> Element {
    let ctx = use_context::<EditorCtx>();
    let input = (ctx.variations.ranges)()
        .get(&dot)
        .copied()
        .unwrap_or_default();
    rsx! {
        div {
            class: "RollRangeField",
            title: "The range {dot} is rolled in",
            onclick: move |event| event.stop_propagation(),
            RangeEnd {
                dot: dot.clone(),
                to: false,
                integer,
                own,
                set: input.from,
                invalid: (required || input.to.is_some()) && input.from.or(own.map(|own| own.from)).is_none(),
            }
            span { class: "hint", "–" }
            RangeEnd {
                dot: dot.clone(),
                to: true,
                integer,
                own,
                set: input.to,
                invalid: (required || input.from.is_some()) && input.to.or(own.map(|own| own.to)).is_none(),
            }
        }
    }
}

/// One end of the range. Empty means the param's own range again.
#[component]
fn RangeEnd(
    dot: String,
    to: bool,
    integer: bool,
    own: Option<RollRange>,
    set: Option<f32>,
    invalid: bool,
) -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut inputs = ctx.variations.ranges;
    let mut selected = ctx.variations.selected;
    let locked = (ctx.variations.rolling)().is_some();
    // the text while typing, so a commit that changes nothing still shows the number again
    let mut typed: Signal<Option<String>> = use_signal(|| None);
    let name = if to { "to" } else { "from" };
    let own_end = own.map(|own| if to { own.to } else { own.from });
    let shown = match set.or(own_end) {
        Some(value) if integer => (value.round() as i64).to_string(),
        Some(value) => format_number(value),
        None => String::new(),
    };
    let text = typed().unwrap_or(shown);

    let label = format!("{dot} {name}");
    let mut commit = move |text: String| {
        let text = text.trim();
        let end = match text.parse::<f32>() {
            _ if text.is_empty() => None,
            Ok(value) if value.is_finite() => Some(value),
            _ => {
                return consume_context::<ActiveConsole>()
                    .error(&format!("'{text}' is not a number"))
            }
        };
        // stays within the param's own range
        let end = end.map(|value| {
            let value = if integer { value.round() } else { value };
            own.map_or(value, |own| value.max(own.from).min(own.to))
        });
        let mut input = inputs.peek().get(&dot).copied().unwrap_or_default();
        match to {
            true => input.to = end,
            false => input.from = end,
        }
        match input == RangeInput::default() {
            true => inputs.write().remove(&dot),
            false => inputs.write().insert(dot.clone(), input),
        };
        // a range is set to roll in it
        if end.is_some() && !selected.peek().contains(&dot) {
            selected.write().insert(dot.clone());
        }
    };

    rsx! {
        input {
            class: if invalid { "input invalid" } else { "input" },
            placeholder: name,
            "aria-label": label,
            disabled: locked,
            value: "{text}",
            oninput: move |event| typed.set(Some(event.value())),
            onchange: move |event| {
                commit(event.value());
                typed.set(None);
            },
            onblur: move |_| typed.set(None),
        }
    }
}
