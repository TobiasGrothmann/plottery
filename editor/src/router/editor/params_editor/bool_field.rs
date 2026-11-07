use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_project::{
    project_param::ProjectParam, project_param_value::ProjectParamValue,
    project_params_list_wrapper::ProjectParamsListWrapper,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::components::ui::Switch;
use crate::router::editor::{
    console_messages::ConsoleMessages, params_editor::param_tree::get_param_value_mut_by_path,
    params_editor::params_editor::trigger_run, project_runner::ProjectRunner,
    running_state::RunningState,
};

#[derive(Props, Clone)]
pub struct BoolFieldProps {
    param: ProjectParam,
    path: Vec<String>,
    project_params: SyncSignal<ProjectParamsListWrapper>,
    project_runner: SyncSignal<Arc<Mutex<ProjectRunner>>>,
    running_state: SyncSignal<RunningState>,
    console: SyncSignal<ConsoleMessages>,
    release: bool,
}

/// `ProjectParam` only compares its schema, so compare the value too or Dioxus will not re-render.
impl PartialEq for BoolFieldProps {
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
pub fn BoolField(mut props: BoolFieldProps) -> Element {
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
                let mut new_params = props.project_params.read().clone();
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
