use dioxus::prelude::*;
use dioxus_logger::tracing;

use super::format_number;

#[derive(PartialEq, Props, Clone)]
pub struct NumberInputProps {
    pub value: f32,
    /// Shows and commits whole numbers only.
    #[props(default)]
    pub integer: bool,
    #[props(default)]
    pub placeholder: String,
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
    rsx! {
        style { { include_str!("widgets.css") } }
        div { class: "NumberInput",
            input {
                placeholder: "{props.placeholder}",
                value: "{shown}",
                onchange: move |event| match event.value().trim().parse::<f32>() {
                    Ok(value) => props.oncommit.call(if integer { value.round() } else { value }),
                    Err(e) => tracing::error!("Error parsing number: {:?}", e),
                },
            }
        }
    }
}
