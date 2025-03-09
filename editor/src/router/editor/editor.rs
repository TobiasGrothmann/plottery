use crate::{
    components::{
        image::{Image, SVGImage},
        loading_spinner::Loading,
        navigation::Navigation,
    },
    router::editor::{
        console::Console,
        editor_console::EditorConsole,
        layer_editor::{layer_editor::LayerEditor, layer_tree_ref::LayerTreeReference},
        params_editor::params_editor::ParamsEditor,
        project_hot_reload::start_hot_reload,
        project_runner::ProjectRunner,
        running_state::RunningState,
    },
    util::format_svg,
};
use bincode::{deserialize, serialize};
use dioxus::prelude::*;
use notify::FsEventWatcher;
use plottery_lib::{rand_range_i, Layer, LayerProps, SampleSettings};
use plottery_project::{project_params_list_wrapper::ProjectParamsListWrapper, Project};
use plottery_server_lib::{plot_setting::PlotSettings, task::send_task};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

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

    let release = true;

    // ui state
    let project_params = use_signal_sync(|| {
        // read params from binary file
        let params_file_path = project().get_editor_params_path();
        match std::fs::read(params_file_path) {
            Ok(params_binary) => {
                deserialize(&params_binary).expect("Failed to deserialize project params")
            }
            Err(_) => ProjectParamsListWrapper::new(vec![]),
        }
    });
    let layer = use_signal_sync(|| {
        let layer_from_file = {
            match std::fs::read(project().get_editor_layer_path()) {
                Ok(layer_binary) => Some(
                    Layer::new_from_binary(&layer_binary).expect("Failed to deserialize layer"),
                ),
                Err(_) => None,
            }
        };
        LayerChangeWrapper {
            layer: layer_from_file,
            change_counter: 0,
        }
    });
    let mut layer_tree_ref = use_signal_sync(move || {
        layer()
            .layer
            .map(|layer| LayerTreeReference::new(&layer, &LayerProps::default()))
    });
    use_effect(move || {
        layer_tree_ref.set(
            layer()
                .layer
                .map(|layer| LayerTreeReference::new(&layer, &LayerProps::default())),
        )
    });
    let mut layer_only_visible_change_counter = use_signal(|| 0);
    let layer_only_visible = use_memo(move || {
        let mut cc = layer_only_visible_change_counter.write();
        let new_count = *cc + 1;
        *cc = new_count;
        let layer = layer().clone();
        match layer_tree_ref() {
            Some(layer_tree_ref) => LayerChangeWrapper {
                layer: Some(layer_tree_ref.filter_layer_by_visibility(&layer.layer.unwrap())),
                change_counter: new_count,
            },
            None => LayerChangeWrapper {
                layer: None,
                change_counter: new_count,
            },
        }
    });

    // console
    let console_change_counter = use_signal_sync(|| 0);
    let console: Signal<EditorConsole, SyncStorage> =
        use_signal_sync(|| EditorConsole::new(console_change_counter));

    // hooks for changes in project
    // params
    use_effect(move || {
        let params_binary =
            serialize(&(project_params())).expect("Failed to serialize project params");
        let params_file_path = project().get_editor_params_path();
        std::fs::write(params_file_path, params_binary)
            .expect("Failed to write project params to file");
    });
    // layer
    let svg = use_memo(move || match &layer_only_visible().layer {
        Some(layer) => Some(layer.to_svg(1.0).to_string()),
        None => None,
    });
    use_effect(move || {
        let layer_path = project().get_editor_layer_path();
        if let Some(layer) = &layer().layer {
            let binary = layer.to_binary().expect("Failed to serialize layer");
            match std::fs::write(layer_path, binary) {
                Ok(_) => (),
                Err(e) => log::error!("Failed to write layer to file: {:?}", e),
            }
        }
        let svg_path = project().get_editor_preview_image_path();
        if let Some(svg) = svg() {
            match std::fs::write(svg_path, svg) {
                Ok(_) => (),
                Err(e) => log::error!("Failed to write .svg to file: {:?}", e),
            }
        }
    });

    // running project
    let mut running_state = use_signal_sync(|| RunningState::Idle);
    let project_runner = use_signal_sync(|| {
        Arc::new(Mutex::new(ProjectRunner::new(
            project().clone(),
            layer,
            project_params,
        )))
    });

    // hot reload
    let mut hot_reload_watcher = use_signal_sync(|| None as Option<FsEventWatcher>);
    let mut hot_reload_join_handle = use_signal_sync(|| None as Option<JoinHandle<()>>);
    let hot_reload_path_to_watch = project().get_cargo_path().unwrap().join("src");
    let hot_reload_is_enabled = move || hot_reload_watcher.read().is_some();

    let hot_reload_button_class = if hot_reload_is_enabled() {
        "active_button"
    } else {
        ""
    };

    let running_state_class = if running_state().is_error() {
        "running_state err_box"
    } else {
        "running_state"
    };

    let icon_folder = if cfg!(target_os = "windows") {
        format_svg(include_bytes!("../../../public/icons/explorer.svg"))
    } else if cfg!(target_os = "macos") {
        format_svg(include_bytes!("../../../public/icons/finder.svg"))
    } else {
        format_svg(include_bytes!("../../../public/icons/linux_folder.svg"))
    };

    rsx! {
        style { { include_str!("./editor.css") } }
        Navigation { page_name: "{project().config.name.clone()}" }

        div { class: "Editor",
            div { class: "plot_header",
                div { class: "open_actions",
                    button { class: "icon_button",
                        onclick: move |_event| {
                            let cargo_dir = project().get_cargo_path().unwrap();
                            std::process::Command::new("code")
                                .arg(cargo_dir)
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        },
                        img { src: "{format_svg(include_bytes!(\"../../../public/icons/vscode.svg\"))}" }
                    }
                    button { class: "icon_button",
                        onclick: move |_event| {
                            opener::reveal(project().dir.clone()).unwrap();
                        },
                        img { src: "{icon_folder}" }
                    }
                    button { class: "icon_button",
                        onclick: move |_event| {
                            let project_dir = project().get_dir();
                            std::process::Command::new("GitKraken")
                                .arg("-p")
                                .arg(project_dir)
                                .spawn()
                                .unwrap()
                                .wait()
                                .unwrap();
                        },
                        img { src: "{format_svg(include_bytes!(\"../../../public/icons/gitkraken.svg\"))}" }
                    }
                }
                div { class: "run_actions",
                    if !matches!(running_state(), RunningState::Idle {}) {
                        div { class: "{running_state_class}",
                            p { "{running_state().get_msg()}" }
                        }
                    }
                    if !hot_reload_is_enabled() {
                        button { class: "img_button",
                            onclick: move |_event| {
                                running_state.set(RunningState::Preparing { msg: "preparing".to_string() });
                                match project_runner().try_lock() {
                                    Ok(mut runner) => runner.trigger_run_project(release, running_state, console),
                                    Err(e) => {
                                        log::error!("Error preparing to run: {:?}", e);
                                        running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                    },
                                }
                            },
                            img { src: "{format_svg(include_bytes!(\"../../../public/icons/play.svg\"))}" }
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
                                    project_runner().clone(),
                                    running_state,
                                    console,
                                );
                                hot_reload_join_handle.set(Some(handle));
                                hot_reload_watcher.set(Some(watcher));
                            }
                        },
                        p { "hot reload" }
                    }
                }
                div { class: "output_actions",
                    button { class: "img_button",
                        onclick: move |_event| {
                            let layer_option = layer().layer.clone();
                            match layer_option {
                                Some(layer) => {
                                    tokio::spawn(async move {
                                        console().info("...sending plot");
                                        let plot_result = send_task(plottery_server_lib::task::Task::Plot {
                                            layer,
                                            sample_settings: SampleSettings::default(),
                                            plot_settings: PlotSettings::default()
                                        }).await;
                                        if plot_result.is_err() {
                                            console().error(format!("failed to send plot: {:?}", plot_result.err().unwrap()).as_str());
                                        }
                                    });
                                },
                                None => {
                                    console().error("cannot send plot: no layer available");
                                }
                            }

                        },
                        p { "send plot" }
                    }
                    button { class: "img_button",
                        onclick: move |_event| {
                            if let Some(svg) = svg() {
                                let path = std::env::temp_dir().join("temp_editor.svg");
                                std::fs::write(path.clone(), svg).unwrap();
                                open::that_in_background(path)
                                        .join()
                                        .expect("Failed to open svg.")
                                        .expect("Failed to open svg.");
                            }
                        },
                        p { "open svg" }
                    }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    ParamsEditor {
                        project_params: project_params,
                        project_runner: project_runner,
                        running_state: running_state,
                        console: console,
                        release: release,
                    }
                }
                div { class: "plot_and_console",
                    div { class: "plot",
                        if project().get_editor_preview_image_path().exists() {
                            if running_state().is_busy() {
                                Loading {}
                            }
                            if let Some(svg) = svg() {
                                SVGImage { svg }
                            }
                            else {
                                Image {
                                    img_path: project().get_editor_preview_image_path().to_str().unwrap().to_string(),
                                    redraw_counter: layer().change_counter,
                                }
                            }
                        } else {
                            div { class: "err_box",
                                p { ".svg could not be found!" }
                            }
                        }
                    }
                    div { class: "console",
                        Console {
                            console: console,
                        }
                    }
                }
                LayerEditor {
                    layer_tree_ref,
                    running_state: running_state,
                }
            }
        }
    }
}
