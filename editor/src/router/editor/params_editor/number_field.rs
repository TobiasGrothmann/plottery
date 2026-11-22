use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::number_input::NumberInput,
};

#[component]
pub fn NumberField(props: LeafProps) -> Element {
    let (value, integer) = match props.param.value {
        ProjectParamValue::Float(value) => (value, false),
        ProjectParamValue::Int(value) => (value as f32, true),
        _ => return rsx! { p { "unexpected value type" } },
    };
    let edit_props = props.clone();
    rsx! {
        NumberInput {
            value,
            integer,
            placeholder: "value",
            aria_label: props.param.formatted_name(),
            oncommit: move |new: f32| {
                edit_props.edit(|value| match value {
                    ProjectParamValue::Float(inner) => *inner = new,
                    ProjectParamValue::Int(inner) => *inner = new.round() as i32,
                    _ => {}
                });
            },
        }
    }
}
