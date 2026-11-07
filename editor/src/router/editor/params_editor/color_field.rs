use dioxus::prelude::*;
use plottery_lib::ColorRgb;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::params_editor::LeafProps;

fn set_color(props: &LeafProps, hex: &str) {
    let Some(color) = ColorRgb::from_hex(hex) else {
        return;
    };
    props.edit(|value| {
        if let ProjectParamValue::Color(inner) = value {
            *inner = color;
        }
    });
}

#[component]
pub fn ColorField(props: LeafProps) -> Element {
    let ProjectParamValue::Color(color) = props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let hex = color.hex();
    let name = color.get_name().name.to_lowercase();
    let picker_props = props.clone();
    let text_props = props.clone();

    rsx! {
        style { { include_str!("color_field.css") } }
        div { class: "ColorField",
            input {
                r#type: "color",
                class: "color_picker",
                value: "{hex}",
                onchange: move |event| set_color(&picker_props, &event.value()),
            }
            input {
                r#type: "text",
                class: "color_hex",
                value: "{hex}",
                onchange: move |event| set_color(&text_props, &event.value()),
            }
            span { class: "color_name", "{name}" }
        }
    }
}
