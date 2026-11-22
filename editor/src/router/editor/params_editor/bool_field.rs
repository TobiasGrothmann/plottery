use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::components::ui::Switch;
use crate::router::editor::params_editor::params_editor::LeafProps;

#[component]
pub fn BoolField(props: LeafProps) -> Element {
    let ProjectParamValue::Bool(checked) = props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let edit_props = props.clone();
    rsx! {
        Switch {
            aria_label: "{props.param.formatted_name()}",
            checked,
            onchange: move |new: bool| {
                edit_props.edit(|value| {
                    if let ProjectParamValue::Bool(inner) = value {
                        *inner = new;
                    }
                });
            },
        }
    }
}
