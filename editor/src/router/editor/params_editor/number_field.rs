use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};

use crate::router::editor::{
    console_messages::ConsoleMessages,
    params_editor::param_tree::get_param_value_mut_by_path,
    project_runner::{trigger_run, ProjectRunner},
    running_state::RunningState,
};

#[derive(Props, Clone)]
pub struct NumberFieldProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<ProjectRunner>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for NumberFieldProps {
    fn eq(&self, other: &Self) -> bool {
        self.param.name == other.param.name
            && self.param.value == other.param.value
            && self.path == other.path
            && self.project_params == other.project_params
            && self.project_runner == other.project_runner
            && self.running_state == other.running_state
            && self.console == other.console
            && self.release == other.release
    }
}

#[component]
pub fn NumberField(mut props: NumberFieldProps) -> Element {
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
                let mut new_params = props.project_params.read().clone();
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
                        Err(e) => {
                            tracing::error!("Error parsing value: {:?}", e);
                            return;
                        }
                    }
                } else {
                    tracing::error!("Param path not found: {:?}", props.path);
                    return;
                }
                props.project_params.set(new_params);
                trigger_run(
                    props.project_runner,
                    props.running_state,
                    props.console,
                    props.release,
                );
            },
        }
    }
}
