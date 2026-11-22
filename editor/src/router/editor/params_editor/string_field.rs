use dioxus::prelude::*;
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::router::editor::params_editor::{
    params_editor::LeafProps, widgets::text_input::TextInput,
};

#[component]
pub fn StringField(props: LeafProps) -> Element {
    let leaf = props.use_handle();
    let ProjectParamValue::String(value) = &props.param.value else {
        return rsx! { p { "unexpected value type" } };
    };
    rsx! {
        TextInput {
            value: value.clone(),
            oncommit: move |text: String| {
                leaf.edit(|value| {
                    if let ProjectParamValue::String(inner) = value {
                        *inner = text;
                    }
                });
            },
        }
    }
}
