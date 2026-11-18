use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_runtime::{project_param::ProjectParam, project_param_value::ProjectParamValue};

use crate::router::editor::{
    editor_ctx::EditorCtx, params_editor::param_tree::get_param_value_mut_by_path,
};

#[derive(Props, Clone)]
pub struct NumberFieldProps {
    param: ProjectParam,
    path: Vec<String>,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for NumberFieldProps {
    fn eq(&self, other: &Self) -> bool {
        self.param.name == other.param.name
            && self.param.value == other.param.value
            && self.path == other.path
    }
}

#[component]
pub fn NumberField(props: NumberFieldProps) -> Element {
    let mut ctx = use_context::<EditorCtx>();
    rsx! {
        input {
            class: "input",
            name: "{props.param.name.clone()}",
            "aria-label": "{props.param.formatted_name()}",
            placeholder: "value",
            required: true,
            value: match &props.param.value {
                ProjectParamValue::Float(val) => val.to_string(),
                ProjectParamValue::Int(val) => val.to_string(),
                ProjectParamValue::Optional(optional) => {
                    match optional.value.as_ref() {
                        ProjectParamValue::Float(val) => val.to_string(),
                        ProjectParamValue::Int(val) => val.to_string(),
                        _ => panic!("Unexpected Error"),
                    }
                }
                _ => panic!("Unexpected Error"),
            },
            onchange: move |event| {
                let mut new_params = ctx.project_params.read().clone();
                if let Some(param_value) = get_param_value_mut_by_path(
                    &mut new_params.list,
                    &props.path,
                ) {
                    let new_val = event.value().parse::<f32>();
                    match new_val {
                        Ok(val) => {
                            match param_value {
                                ProjectParamValue::Float(inner) => *inner = val,
                                ProjectParamValue::Int(inner) => *inner = val.round() as i32,
                                ProjectParamValue::Optional(optional) => {
                                    match optional.value.as_mut() {
                                        ProjectParamValue::Float(inner) => *inner = val,
                                        ProjectParamValue::Int(inner) => *inner = val.round() as i32,
                                        _ => panic!("Unexpected Error"),
                                    }
                                }
                                _ => panic!("Unexpected Error"),
                            }
                        }
                        Err(_) => {
                            ctx.console.peek().error(&format!("'{}' is not a number", event.value()));
                            return;
                        }
                    }
                } else {
                    tracing::error!("Param path not found: {:?}", props.path);
                    return;
                }
                ctx.project_params.set(new_params);
                ctx.trigger_run();
            },
        }
    }
}
