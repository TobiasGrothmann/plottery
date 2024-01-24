use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use dioxus_std::utils::rw::{use_rw, UseRw};
use notify::{Config, FsEventWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use path_absolutize::Absolutize;
use plottery_project::Project;
use std::path::PathBuf;
use tokio::task::JoinHandle;

use crate::{components::image::Image, project_runner::project_runner::ProjectRunner};

fn get_svg_path(project: &Project) -> PathBuf {
    project.get_preview_image_path()
}

fn run_project(
    project: &Project,
    run_counter: UseRw<u32>,
    busy_running: UseRw<bool>,
) -> JoinHandle<()> {
    let project = project.clone();
    let svg_path = get_svg_path(&project);
    tokio::spawn(async move {
        busy_running.write(true).unwrap();
        match project.compile(true) {
            Ok(()) => {}
            Err(e) => {
                log::error!("Error compiling project {}", e);
                busy_running.write(false).unwrap();
                return;
            }
        }
        let new_layer = project.run_code(true);
        match new_layer {
            Ok(new_layer) => {
                match new_layer.write_svg(svg_path, 1.0) {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("Error writing svg {}", e);
                        busy_running.write(false).unwrap();
                        return;
                    }
                };
                let new_run_counter = *run_counter.read().unwrap() + 1;
                run_counter.write(new_run_counter).unwrap();
            }
            Err(e) => {
                log::error!("Error running code: {}", e)
            }
        }
        busy_running.write(false).unwrap();
    })
}

fn start_hot_reload(
    p: PathBuf,
    project: &Project,
    run_counter: UseRw<u32>,
    busy_running: UseRw<bool>,
) -> (JoinHandle<()>, FsEventWatcher) {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default()).unwrap();
    watcher.watch(&p, RecursiveMode::Recursive).unwrap();

    let project = project.clone();
    let run_counter = run_counter.clone();
    let busy_running = busy_running.clone();

    let handle = tokio::spawn(async move {
        let last_event_time = std::time::Instant::now();
        for event in rx {
            match event {
                Ok(event) => {
                    if last_event_time.elapsed().as_millis() < 100 {
                        continue;
                    }

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

                    run_project(&project, run_counter.to_owned(), busy_running.to_owned());
                }
                Err(e) => log::error!("watch error: {:?}", e),
            }
        }
    });

    (handle, watcher)
}

#[component]
pub fn Editor(cx: Scope, project_path: String) -> Element {
    let project_rw = use_rw(cx, || {
        let p = PathBuf::from(project_path.clone());
        Project::load_from_file(p).unwrap()
    });
    let busy_compiling = use_rw(cx, || false);
    let busy_running = use_rw(cx, || false);

    let run_counter = use_rw(cx, || 0);

    let hot_reload_enabled = use_state(cx, || false);
    let hot_reload_watcher = use_state(cx, || None as Option<FsEventWatcher>);

    let hot_reload_join_handle = use_state(cx, || None as Option<JoinHandle<()>>);
    let run_join_handle = use_state(cx, || None as Option<JoinHandle<()>>);

    let project_runner = use_state(cx, || {
        ProjectRunner::new(project_rw.read().unwrap().clone())
    });

    // clones for closures
    let project_name = project_rw.read().unwrap().config.name.clone();
    let project: Project = project_rw.read().unwrap().clone();
    let project_run_clone = project.clone();
    let project_hot_reload_clone = project.clone();
    let run_counter_run_clone = run_counter.clone();
    let run_counter_hot_reload_clone = run_counter.clone();
    let busy_running_hot_reload_clone = busy_running.clone();

    cx.render(rsx! {
        style { include_str!("./editor.css") }

        div { class: "Editor",
            div { class: "plot_header",
                button { class: "img_button",
                    onclick: move |_event| {
                        let nav = use_navigator(cx);
                        nav.go_back();
                    },
                    img { src: "icons/back.svg" }
                }
                h1 {
                    "{project_name}"
                }
                div { class: "action_buttons",
                    if *busy_running.read().unwrap() {
                        cx.render(rsx!("busy running"))
                    }
                    button { class: "img_button",
                        onclick: move |_event| {
                            // // cancel previous run
                            // if let Some(handle) = run_join_handle.get() {
                            //     handle.abort();
                            // }
                            // // start running
                            // let join_handle = run_project(&project_run_clone, run_counter_run_clone.to_owned(), busy_running.to_owned());
                            // run_join_handle.set(Some(join_handle));
                            project_runner.with_mut(|runner| {
                                runner.trigger_run_project(true);
                            })
                        },
                        img { src: "icons/play.svg" }
                    }
                    // button { class: "img_button",
                    //     onclick: move |_event| {
                    //         if let Some(handle) = hot_reload_join_handle.get() {
                    //             handle.abort();
                    //         }
                    //         hot_reload_watcher.set(None);

                    //         let path = project_hot_reload_clone.get_cargo_path().unwrap().join("src");
                    //         let (join_handle, watcher) = start_hot_reload(path, &project_hot_reload_clone, run_counter_hot_reload_clone.to_owned(), busy_running_hot_reload_clone.to_owned());
                    //         hot_reload_join_handle.set(Some(join_handle));
                    //         hot_reload_watcher.set(Some(watcher));
                    //     },
                    //     "start hot reload"
                    // }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    p { "Parameters" } 
                }
                div { class: "plot",
                    if get_svg_path(&project).exists() {
                        cx.render(rsx!(
                            Image {
                                img_path: get_svg_path(&project).absolutize().unwrap().to_string_lossy().to_string(),
                                redraw_counter: *run_counter.read().unwrap()
                            }
                        ))
                    } else {
                        cx.render(rsx!(
                            div { class: "err_box",
                                p { "SVG could not be found!" }
                            }
                        ))
                    }
                }
            }
        }
    })
}
