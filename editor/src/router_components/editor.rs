use crate::{
    components::navigation::Navigation,
    router_components::editor_components::{
        params_editor::ParamsEditor, project_hot_reload::start_hot_reload,
        project_runner::ProjectRunner, running_state::RunningState,
    },
    util::format_svg,
};
use dioxus::prelude::*;
use notify::FsEventWatcher;
use path_absolutize::Absolutize;
use plottery_lib::Layer;
use plottery_project::{Project, ProjectParamsListWrapper};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::components::image::Image;

fn get_svg_path(project: &Project) -> PathBuf {
    project.get_preview_image_path()
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

#[component]
pub fn Editor(project_path: String) -> Element {
    let project = use_signal(|| {
        let p = PathBuf::from(project_path.clone());
        Project::load_from_file(p).unwrap()
    });

    // ui state
    let mut project_params = use_signal_sync(|| ProjectParamsListWrapper::new(vec![]));
    let layer = use_signal_sync(|| LayerChangeWrapper {
        layer: None,
        change_counter: 0,
    });

    // running project
    let mut running_state = use_signal_sync(|| RunningState::Idle);
    let project_runner = use_signal_sync(|| {
        Arc::new(Mutex::new(ProjectRunner::new(
            project.read().clone(),
            layer,
            project_params,
        )))
    });

    // hot reload
    let mut hot_reload_watcher = use_signal_sync(|| None as Option<FsEventWatcher>);
    let mut hot_reload_join_handle = use_signal_sync(|| None as Option<JoinHandle<()>>);
    let hot_reload_path_to_watch = project.read().get_cargo_path().unwrap().join("src");
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
                    ParamsEditor {
                        project_params: project_params,
                        project_runner: project_runner,
                        running_state: running_state,
                        release: release,
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
