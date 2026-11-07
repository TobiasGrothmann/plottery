use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::slider_input::SliderInput,
};

#[derive(Clone, Copy)]
struct Ranged {
    val: f32,
    min: f32,
    max: f32,
    is_int: bool,
}

impl Ranged {
    fn from_value(value: &ProjectParamValue) -> Option<Self> {
        match value {
            ProjectParamValue::FloatRanged { val, min, max } => Some(Ranged {
                val: *val,
                min: *min,
                max: *max,
                is_int: false,
            }),
            ProjectParamValue::IntRanged { val, min, max } => Some(Ranged {
                val: *val as f32,
                min: *min as f32,
                max: *max as f32,
                is_int: true,
            }),
            _ => None,
        }
    }
}

fn set_ranged_value(param_value: &mut ProjectParamValue, new_val: f32) {
    match param_value {
        ProjectParamValue::FloatRanged { val, min, max } => *val = new_val.clamp(*min, *max),
        ProjectParamValue::IntRanged { val, min, max } => {
            *val = (new_val.round() as i32).clamp(*min, *max)
        }
        _ => {}
    }
}

#[component]
pub fn Slider(props: LeafProps) -> Element {
    let Some(ranged) = Ranged::from_value(&props.param.value) else {
        return rsx! { p { "unexpected value type" } };
    };
    let edit_props = props.clone();
    rsx! {
        SliderInput {
            value: ranged.val,
            min: ranged.min,
            max: ranged.max,
            integer: ranged.is_int,
            oncommit: move |value: f32| edit_props.edit(|param| set_ranged_value(param, value)),
        }
    }
}
