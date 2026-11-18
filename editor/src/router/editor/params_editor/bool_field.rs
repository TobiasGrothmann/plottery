use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_runtime::{project_param::ProjectParam, project_param_value::ProjectParamValue};

use crate::components::ui::Switch;
use crate::router::editor::{
    editor_ctx::EditorCtx, params_editor::param_tree::get_param_value_mut_by_path,
};

#[derive(Props, Clone)]
pub struct BoolFieldProps {
    param: ProjectParam,
    path: Vec<String>,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for BoolFieldProps {
    fn eq(&self, other: &Self) -> bool {
        self.param.name == other.param.name
            && self.param.value == other.param.value
            && self.path == other.path
    }
}

#[component]
pub fn BoolField(props: BoolFieldProps) -> Element {
    let mut ctx = use_context::<EditorCtx>();
    rsx! {
        Switch {
            aria_label: "{props.param.formatted_name()}",
            checked: match &props.param.value {
                ProjectParamValue::Bool(val) => *val,
                ProjectParamValue::Optional(optional) => {
                    match optional.value.as_ref() {
                        ProjectParamValue::Bool(val) => *val,
                        _ => panic!("Unexpected Error"),
                    }
                }
                _ => panic!("Unexpected Error"),
            },
            onchange: move |new_val: bool| {
                let mut new_params = ctx.project_params.read().clone();
                if let Some(param_value) = get_param_value_mut_by_path(
                    &mut new_params.list,
                    &props.path,
                ) {
                    match param_value {
                        ProjectParamValue::Bool(val) => *val = new_val,
                        ProjectParamValue::Optional(optional) => {
                            match optional.value.as_mut() {
                                ProjectParamValue::Bool(val) => *val = new_val,
                                _ => panic!("Unexpected Error"),
                            }
                        }
                        _ => panic!("Unexpected Error"),
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
