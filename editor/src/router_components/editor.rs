use crate::{components::navigation::Navigation, util::format_svg};
use dioxus::prelude::*;
use notify::{Config, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use path_absolutize::Absolutize;
use plottery_lib::Layer;
use plottery_project::{Project, ProjectParamValue, ProjectParamsListWrapper};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{components::image::Image, util::ProjectRunner};

fn get_svg_path(project: &Project) -> PathBuf {
    project.get_preview_image_path()
}

fn start_hot_reload(
    path_to_watch: PathBuf,
    release: bool,
    project_runner: Arc<Mutex<ProjectRunner>>,
    running_state: SyncSignal<RunningState>,
) -> (JoinHandle<()>, FsEventWatcher) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher
        .watch(&path_to_watch, RecursiveMode::Recursive)
        .unwrap();

    let hot_reload_handle = tokio::spawn(async move {
        for event in rx {
            match event {
                Ok(event) => {
                    let ignore_list = [".DS_Store"];
                    let changed_paths = event.paths.iter().filter(|p| {
                        p.file_name()
                            .map(|s| !ignore_list.contains(&s.to_str().unwrap()))
                            .unwrap_or(false)
                    });
                    if changed_paths.count() == 0 {
                        continue;
                    }

                    let hot_reload_required = match event.kind {
                        notify::EventKind::Any => false,
                        notify::EventKind::Access(_) => false,
                        notify::EventKind::Create(_) => true,
                        notify::EventKind::Modify(modifyKind) => match modifyKind {
                            notify::event::ModifyKind::Any => false,
                            notify::event::ModifyKind::Data(_) => true,
                            notify::event::ModifyKind::Metadata(_) => false,
                            notify::event::ModifyKind::Name(_) => true,
                            notify::event::ModifyKind::Other => true,
                        },
                        notify::EventKind::Remove(_) => true,
                        notify::EventKind::Other => false,
                    };
                    if !hot_reload_required {
                        continue;
                    }

                    log::info!("Hot reload triggered");
                    project_runner
                        .lock()
                        .await
                        .trigger_run_project(release, running_state);
                }
                Err(e) => log::error!("Hot reload error: {:?}", e),
            }
        }
        log::info!("Hot reload thread finished");
    });

    (hot_reload_handle, watcher)
}

#[derive(Debug, Clone)]
pub struct LayerChangeWrapper {
    pub layer: Option<Layer>,
    pub change_counter: u32,
}
impl PartialEq for LayerChangeWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.change_counter == other.change_counter
    }
}

#[derive(Debug, Clone)]
pub enum RunningState {
    Idle,
    Preparing { msg: String },
    StartingBuild { msg: String },
    Building { msg: String },
    BuildFailed { msg: String },
    BuildKilled { msg: String },
    StartingGetParams { msg: String },
    GetParams { msg: String },
    GetParamsFailed { msg: String },
    GetParamsKilled { msg: String },
    StartingRun { msg: String },
    Running { msg: String },
    RunFailed { msg: String },
    RunKilled { msg: String },
    Updating { msg: String },
}
impl RunningState {
    pub fn is_busy(&self) -> bool {
        !matches!(self, RunningState::Idle {})
    }
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            RunningState::BuildFailed { .. }
                | RunningState::RunFailed { .. }
                | RunningState::GetParamsFailed { .. }
        )
    }
    pub fn get_msg(&self) -> String {
        match self {
            RunningState::Idle {} => "".to_string(),
            RunningState::Preparing { msg } => msg.clone(),
            RunningState::StartingBuild { msg } => msg.clone(),
            RunningState::Building { msg } => msg.clone(),
            RunningState::BuildFailed { msg } => msg.clone(),
            RunningState::BuildKilled { msg } => msg.clone(),
            RunningState::StartingRun { msg } => msg.clone(),
            RunningState::Running { msg } => msg.clone(),
            RunningState::RunFailed { msg } => msg.clone(),
            RunningState::RunKilled { msg } => msg.clone(),
            RunningState::Updating { msg } => msg.clone(),
            RunningState::StartingGetParams { msg } => msg.clone(),
            RunningState::GetParams { msg } => msg.clone(),
            RunningState::GetParamsFailed { msg } => msg.clone(),
            RunningState::GetParamsKilled { msg } => msg.clone(),
        }
    }
}

#[component]
pub fn Editor(project_path: String) -> Element {
    let project = use_signal(|| {
        let p = PathBuf::from(project_path.clone());
        Project::load_from_file(p).unwrap()
    });

    let mut running_state = use_signal_sync(|| RunningState::Idle);

    let mut hot_reload_watcher = use_signal_sync(|| None as Option<FsEventWatcher>);
    let mut hot_reload_join_handle = use_signal_sync(|| None as Option<JoinHandle<()>>);
    let hot_reload_path_to_watch = project.read().get_cargo_path().unwrap().join("src");

    let layer = use_signal_sync(|| LayerChangeWrapper {
        layer: None,
        change_counter: 0,
    });
    let mut params = use_signal_sync(|| ProjectParamsListWrapper::new(vec![]));

    let project_runner = use_signal_sync(|| {
        Arc::new(Mutex::new(ProjectRunner::new(
            project.read().clone(),
            layer,
            params,
        )))
    });

    let hot_reload_is_enabled = move || hot_reload_watcher.read().is_some();

    use_resource(move || async move {
        let new_layer = layer.read().clone().layer;
        if let Some(new_layer) = new_layer {
            let svg_path = get_svg_path(&project.read().clone());
            match new_layer.write_svg(svg_path, 1.0) {
                Ok(_) => log::info!("SVG updated"),
                Err(e) => {
                    log::error!("Error writing svg {}", e);
                }
            };
        };
    })
    .unwrap();

    let hot_reload_button_class = if hot_reload_is_enabled() {
        "active_button"
    } else {
        ""
    };

    let running_state_class = if running_state.read().is_error() {
        "running_state err_box"
    } else {
        "running_state"
    };

    let release = true;

    rsx! {
        style { { include_str!("./editor.css") } }
        Navigation { page_name: "{project.read().config.name.clone()}" }

        div { class: "Editor",
            div { class: "plot_header",
                div { class: "action_buttons",
                    if running_state.read().is_busy() {
                        div { class: "{running_state_class}",
                            p { "{running_state.read().get_msg()}" }
                        }
                    }
                    if !hot_reload_is_enabled() {
                        button { class: "img_button",
                            onclick: move |_event| {
                                running_state.set(RunningState::Preparing { msg: "preparing".to_string() });
                                match project_runner.read().try_lock() {
                                    Ok(mut runner) => runner.trigger_run_project(release, running_state),
                                    Err(e) => {
                                        log::error!("Error preparing to run: {:?}", e);
                                        running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                    },
                                }
                            },
                            img { src: "{format_svg(include_bytes!(\"../../public/icons/play.svg\"))}" }
                        }
                    }
                    button { class: "img_button {hot_reload_button_class}",
                        onclick: move |_event| {
                            if hot_reload_is_enabled() {
                                // Disable hot reload
                                hot_reload_watcher.set(None);
                                hot_reload_join_handle.set(None);
                            } else {
                                // Enable hot reload
                                let (handle, watcher) = start_hot_reload(
                                    hot_reload_path_to_watch.clone(),
                                    release,
                                    project_runner.read().clone(),
                                    running_state,
                                );
                                hot_reload_join_handle.set(Some(handle));
                                hot_reload_watcher.set(Some(watcher));
                            }
                        },
                        p { "hot reload" }
                    }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    for param in params.read().list.iter().cloned() {
                        div { class: "param",
                            p { "{param.name.clone()}" }

                            match param.value {
                                ProjectParamValue::Float(_) | ProjectParamValue::Int(_) => {
                                    // value
                                    rsx! {
                                        input {
                                            name: "{param.name.clone()}",
                                            placeholder: "value",
                                            required: true,
                                            value: match param.value {
                                                ProjectParamValue::Float(val) => val.to_string(),
                                                ProjectParamValue::Int(val) => val.to_string(),
                                                _ => panic!("Unexpected Error"),
                                            },
                                            onchange: move |event| {
                                                let mut new_params = params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == param.name.clone() {
                                                        let new_val = event.value().parse().unwrap();
                                                        match param_field.value {
                                                            ProjectParamValue::Float(_) => param_field.value.set_f32(new_val),
                                                            ProjectParamValue::Int(_) => param_field.value.set_i32(new_val.round() as i32),
                                                            _ => panic!("Unexpected Error"),
                                                        }
                                                    }
                                                }
                                                params.set(new_params);
                                                match project_runner.read().try_lock() {
                                                    Ok(mut runner) => runner.trigger_run_project(release, running_state),
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                                ProjectParamValue::FloatRanged { .. } | ProjectParamValue::IntRanged { .. } => {
                                    // slider
                                    rsx! {
                                        input {
                                            class: "slider",
                                            name: "{param.name.clone()}",
                                            required: true,
                                            r#type: "range",
                                            step: "0.01",
                                            min: match param.value {
                                                ProjectParamValue::FloatRanged { val: _, min, max: _ } => min.to_string(),
                                                ProjectParamValue::IntRanged { val: _, min, max: _ } => min.to_string(),
                                                _ => panic!("Unexpected Error"),
                                            },
                                            max: match param.value {
                                                ProjectParamValue::FloatRanged { val: _, min: _, max } => max.to_string(),
                                                ProjectParamValue::IntRanged { val: _, min: _, max } => max.to_string(),
                                                _ => panic!("Unexpected Error"),
                                            },
                                            value: match param.value {
                                                ProjectParamValue::FloatRanged { val, min: _, max: _ } => val.to_string(),
                                                ProjectParamValue::IntRanged { val, min: _, max: _ } => val.to_string(),
                                                _ => panic!("Unexpected Error"),
                                            },
                                            onchange: move |event| {
                                                let mut new_params = params.read().clone();
                                                for param_field in new_params.list.iter_mut() {
                                                    if param_field.name == param.name.clone() {
                                                        let new_val = event.value().parse().unwrap();
                                                        match param_field.value {
                                                            ProjectParamValue::FloatRanged { .. } => param_field.value.set_f32(new_val),
                                                            ProjectParamValue::IntRanged { .. } => param_field.value.set_i32(new_val.round() as i32),
                                                            _ => panic!("Unexpected Error"),
                                                        }
                                                    }
                                                }
                                                params.set(new_params);
                                                match project_runner.read().try_lock() {
                                                    Ok(mut runner) => runner.trigger_run_project(release, running_state),
                                                    Err(e) => {
                                                        log::error!("Error preparing to run: {:?}", e);
                                                        running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                                    },
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "plot",
                    if get_svg_path(&project.read()).exists() {
                        Image {
                            img_path: get_svg_path(&project.read()).absolutize().unwrap().to_string_lossy().to_string(),
                            redraw_counter: layer.read().change_counter,
                        }
                    } else {
                        div { class: "err_box",
                            p { "SVG could not be found!" }
                        }
                    }
                }
            }
        }
    }
}
