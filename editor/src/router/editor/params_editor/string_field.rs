use dioxus::prelude::*;
use plottery_project::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::text_input::TextInput,
};

#[component]
pub fn StringField(props: LeafProps) -> Element {
    let ProjectParamValue::String(value) = &props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    let edit_props = props.clone();
    rsx! {
        TextInput {
            value: value.clone(),
            oncommit: move |text: String| {
                edit_props.edit(|value| {
                    if let ProjectParamValue::String(inner) = value {
                        *inner = text;
                    }
                });
            },
        }
    }
}
