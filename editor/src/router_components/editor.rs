use crate::{components::navigation::Navigation, util::format_svg};
use dioxus::prelude::*;
use notify::{Config, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use path_absolutize::Absolutize;
use plottery_lib::Layer;
use plottery_project::Project;
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{components::image::Image, util::ProjectRunner};

fn get_svg_path(project: &Project) -> PathBuf {
    project.get_preview_image_path()
}

fn start_hot_reload(
    path_to_watch: PathBuf,
    project_runner: Arc<Mutex<ProjectRunner>>,
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
                    project_runner.lock().await.trigger_run_project(true);
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

#[component]
pub fn Editor(project_path: String) -> Element {
    let project = use_signal(|| {
        let p = PathBuf::from(project_path.clone());
        Project::load_from_file(p).unwrap()
    });
    let busy_running = use_signal(|| false);

    let hot_reload_enabled = use_signal_sync(|| false);
    let mut hot_reload_watcher = use_signal_sync(|| None as Option<FsEventWatcher>);
    let mut hot_reload_join_handle = use_signal_sync(|| None as Option<JoinHandle<()>>);
    let hot_reload_path_to_watch = project.read().get_cargo_path().unwrap().join("src");

    let layer = use_signal_sync(|| LayerChangeWrapper {
        layer: None,
        change_counter: 0,
    });
    let project_runner = use_signal_sync(|| {
        Arc::new(Mutex::new(ProjectRunner::new(
            project.read().clone(),
            layer.clone(),
        )))
    });

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

    rsx! {
        style { { include_str!("./editor.css") } }
        Navigation { page_name: "{project.read().config.name.clone()}" }

        div { class: "Editor",
            div { class: "plot_header",
                div { class: "action_buttons",
                    if *busy_running.read() {
                        "busy running"
                    }
                    button { class: "img_button",
                        onclick: move |_event| {
                            project_runner.read().try_lock().unwrap().trigger_run_project(false);
                        },
                        img { src: "{format_svg(include_bytes!(\"../../public/icons/play.svg\"))}" }
                    }
                    button { class: "img_button",
                        onclick: move |_event| {
                            let (hot_reload_handle, watcher) = start_hot_reload(hot_reload_path_to_watch.clone(), project_runner.read().clone());
                            hot_reload_join_handle.set(Some(hot_reload_handle));
                            hot_reload_watcher.set(Some(watcher));
                        },
                        p { "Enable Hot Reload" }
                    }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    p { "Parameters" }
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
