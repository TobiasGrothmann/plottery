use dioxus::prelude::*;

use super::format_number;
use crate::router::editor::console_messages::ActiveConsole;

#[derive(PartialEq, Props, Clone)]
pub struct NumberInputProps {
    pub value: f32,
    /// Shows and commits whole numbers only.
    #[props(default)]
    pub integer: bool,
    #[props(default)]
    pub placeholder: String,
    /// Short label drawn inside the input, e.g. `x`.
    #[props(default)]
    pub affix: Option<String>,
    #[props(default)]
    pub aria_label: Option<String>,
    pub oncommit: EventHandler<f32>,
}

/// Text input for a number, commits on Enter or blur.
#[component]
pub fn NumberInput(props: NumberInputProps) -> Element {
    let shown = if props.integer {
        (props.value.round() as i64).to_string()
    } else {
        format_number(props.value)
    };
    let integer = props.integer;
    let aria_label = props.aria_label.clone().or(props.affix.clone());
    let input = rsx! {
        input {
            class: "input",
            placeholder: "{props.placeholder}",
            "aria-label": aria_label,
            value: "{shown}",
            onchange: move |event| match event.value().trim().parse::<f32>() {
                Ok(value) => props.oncommit.call(if integer { value.round() } else { value }),
                Err(_) => consume_context::<ActiveConsole>()
                    .error(&format!("'{}' is not a number", event.value())),
            },
        }
    };
    match props.affix.clone() {
        Some(affix) => rsx! {
            label { class: "affix",
                span { "{affix}" }
                {input}
            }
        },
        None => input,
    }
}
