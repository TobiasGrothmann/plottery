use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::number_input::NumberInput,
};

#[component]
pub fn V2Field(props: LeafProps) -> Element {
    let ProjectParamValue::V2(v) = props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let x_props = props.clone();
    let y_props = props.clone();

    rsx! {
        div { class: "V2Field", style: "display: flex; align-items: center; gap: var(--gap-0);",
            span { "x" }
            NumberInput {
                value: v.x,
                oncommit: move |x: f32| x_props.edit(|value| {
                    if let ProjectParamValue::V2(v) = value { v.x = x; }
                }),
            }
            span { "y" }
            NumberInput {
                value: v.y,
                oncommit: move |y: f32| y_props.edit(|value| {
                    if let ProjectParamValue::V2(v) = value { v.y = y; }
                }),
            }
        }
    }
}
