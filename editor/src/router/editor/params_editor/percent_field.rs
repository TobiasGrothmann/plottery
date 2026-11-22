use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps,
    widgets::{format_number, slider_input::SliderInput},
};

/// Slider in whole percents.
#[component]
pub fn PercentField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::Percent { val, min, max } = props.param.value else {
        return rsx! {
            p { "unexpected value type" }
        };
    };
    // value while the slider is dragged, so the label follows before the commit
    let mut live = use_signal(|| None::<f32>);
    let shown = format_number(live().unwrap_or(val));
    rsx! {
        SliderInput {
            value: val,
            min,
            max,
            integer: true,
            label: "{shown}%",
            oninput: move |value: f32| live.set(Some(value)),
            oncommit: move |value: f32| {
                // a click that moved nothing keeps a value with decimals as it is
                if live.take().is_none() {
                    return;
                }
                leaf.edit(|param| {
                    if let ProjectParamValue::Percent { val, min, max } = param {
                        *val = value.clamp(*min, *max);
                    }
                });
            },
        }
    }
}
