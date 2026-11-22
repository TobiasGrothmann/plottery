use dioxus::prelude::*;
use plottery_lib::ColorRgb;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::params_editor::{LeafHandle, LeafProps};

fn set_color(leaf: LeafHandle, hex: &str) {
    let Some(color) = ColorRgb::from_hex(hex) else {
        return;
    };
    leaf.edit(|value| {
        if let ProjectParamValue::Color(inner) = value {
            *inner = color;
        }
    });
}

#[component]
pub fn ColorField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::Color(color) = props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    let hex = color.hex();
    let name = color.name().name.to_lowercase();

    rsx! {
        div { class: "hrow",
            input {
                r#type: "color",
                class: "color-swatch",
                "aria-label": "Pick color",
                value: "{hex}",
                onchange: move |event| set_color(leaf, &event.value()),
            }
            input {
                r#type: "text",
                class: "input color-hex",
                "aria-label": "Hex color",
                value: "{hex}",
                onchange: move |event| set_color(leaf, &event.value()),
            }
            span { class: "hint", "{name}" }
        }
    }
}
