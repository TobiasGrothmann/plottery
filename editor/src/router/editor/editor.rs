use crate::{
    components::{image::*, loading_spinner::Loading, navigation::Navigation},
    router::editor::{
        console::Console,
        console_messages::ConsoleMessages,
        external_editor_button::ExternalEditorButton,
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
use dioxus_logger::tracing;
use notify::FsEventWatcher;
use plottery_lib::{Layer, LayerPropsInheritable, SampleSettings};
use plottery_project::{project_params_list_wrapper::ProjectParamsListWrapper, Project};
use plottery_server_lib::{plot_setting::PlotSettings, task::send_task};
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Mutex, task::JoinHandle};

fn is_gitkraken_installed() -> bool {
    which::which("GitKraken").is_ok()
}

fn is_vscode_installed() -> bool {
    which::which("code").is_ok()
}

fn is_zed_installed() -> bool {
    which::which("zed").is_ok()
}

fn is_cursor_installed() -> bool {
    which::which("cursor").is_ok()
}

fn is_sublime_installed() -> bool {
    which::which("subl").is_ok()
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
pub fn Editor(project_path: Vec<String>) -> Element {
    let project = use_signal(|| {
        let path_str = format!("/{}", project_path.join("/"));
        let p = PathBuf::from(path_str);
        Project::load_from_file(p).expect("Failed to load project from file")
    });

    let release = true;

    let gitkraken_installed = use_signal(is_gitkraken_installed);
    let vscode_installed = use_signal(is_vscode_installed);
    let zed_installed = use_signal(is_zed_installed);
    let cursor_installed = use_signal(is_cursor_installed);
    let sublime_installed = use_signal(is_sublime_installed);

    // console
    let console_change_counter = use_signal_sync(|| 0);
    let console: Signal<ConsoleMessages, SyncStorage> =
        use_signal_sync(|| ConsoleMessages::new(console_change_counter));
    let mut console_expanded = use_signal(|| false);

    // ui state
    let project_params = use_signal_sync(|| {
        // read params from binary file
        let params_file_path = project().get_editor_params_path();
        match std::fs::read(params_file_path) {
            Ok(params_binary) => match deserialize(&params_binary) {
                Ok(params) => ProjectParamsListWrapper::new(params),
                Err(err) => {
                    console()
                        .error(format!("failed to deserialize project params: {:?}", err).as_str());
                    ProjectParamsListWrapper::new(vec![])
                }
            },
            Err(_) => ProjectParamsListWrapper::new(vec![]),
        }
    });
    let layer = use_signal_sync(|| {
        let layer_from_file = {
            match std::fs::read(project().get_editor_layer_path()) {
                Ok(layer_binary) => match Layer::new_from_binary(&layer_binary) {
                    Ok(layer) => Some(layer),
                    Err(err) => {
                        console().error(
                            format!("failed to deserialize layer from binary: {:?}", err).as_str(),
                        );
                        None
                    }
                },
                Err(err) => {
                    console().error(format!("failed to read layer from file: {:?}", err).as_str());
                    None
                }
            }
        };
        LayerChangeWrapper {
            layer: layer_from_file,
            change_counter: 0,
        }
    });
    let mut layer_tree_ref = use_signal_sync(move || None);
    use_effect(move || {
        layer_tree_ref.set(
            layer()
                .layer
                .map(|layer| LayerTreeReference::new(&layer, &LayerPropsInheritable::default())),
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
    let svg = use_memo(move || {
        layer_only_visible()
            .layer
            .as_ref()
            .map(|layer| layer.to_svg(1.0).to_string())
    });
    use_effect(move || {
        let layer_path = project().get_editor_layer_path();
        if let Some(layer) = &layer().layer {
            let binary = layer.to_binary().expect("Failed to serialize layer");
            match std::fs::write(layer_path, binary) {
                Ok(_) => (),
                Err(e) => tracing::error!("Failed to write layer to file: {:?}", e),
            }
        }
        let svg_path = project().get_editor_preview_image_path();
        if let Some(svg) = svg() {
            match std::fs::write(svg_path, svg) {
                Ok(_) => (),
                Err(e) => tracing::error!("Failed to write .svg to file: {:?}", e),
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
        Navigation { page_name: "{project().config.name.clone()}", body: rsx! {} }

        div { class: "Editor",
            div { class: "plot_header",
                div { class: "open_actions",
                    {
                        let cargo_dir = use_memo(move || project().get_cargo_path().unwrap());
                        let project_dir = use_memo(move || project().get_dir());
                        rsx! {
                            if vscode_installed() {
                                ExternalEditorButton {
                                    editor_name: "VS Code".to_string(),
                                    editor_command: "code".to_string(),
                                    icon_svg: include_bytes!("../../../public/icons/vscode.svg"),
                                    project_dir: cargo_dir,
                                }
                            }
                            if zed_installed() {
                                ExternalEditorButton {
                                    editor_name: "Zed".to_string(),
                                    editor_command: "zed".to_string(),
                                    icon_svg: include_bytes!("../../../public/icons/zed.svg"),
                                    project_dir: cargo_dir,
                                }
                            }
                            if cursor_installed() {
                                ExternalEditorButton {
                                    editor_name: "Cursor".to_string(),
                                    editor_command: "cursor".to_string(),
                                    icon_svg: include_bytes!("../../../public/icons/cursor.svg"),
                                    project_dir: cargo_dir,
                                }
                            }
                            if sublime_installed() {
                                ExternalEditorButton {
                                    editor_name: "Sublime Text".to_string(),
                                    editor_command: "subl".to_string(),
                                    icon_svg: include_bytes!("../../../public/icons/sublime-text.svg"),
                                    project_dir: cargo_dir,
                                }
                            }
                            button { class: "icon_button",
                                onclick: move |event| {
                                    open::that_in_background(project().dir).join().unwrap().unwrap();
                                    event.stop_propagation();
                                },
                                img { src: "{icon_folder}" }
                            }
                            if gitkraken_installed() {
                                ExternalEditorButton {
                                    editor_name: "GitKraken".to_string(),
                                    editor_command: "GitKraken".to_string(),
                                    icon_svg: include_bytes!("../../../public/icons/gitkraken.svg"),
                                    project_dir: project_dir,
                                    extra_args: vec!["-p".to_string()],
                                }
                            }
                        }
                    }
                }
                div { class: "run_actions",
                    if !matches!(running_state(), RunningState::Idle) {
                        div { class: "{running_state_class}",
                            p { "{running_state().get_msg()}" }
                        }
                    }
                    if !hot_reload_is_enabled() {
                        button { class: "img_button",
                            onclick: move |event| {
                                running_state.set(RunningState::Preparing { msg: "preparing".to_string() });
                                match project_runner().try_lock() {
                                    Ok(mut runner) => runner.trigger_run_project(release, running_state, console),
                                    Err(e) => {
                                        tracing::error!("Error preparing to run: {:?}", e);
                                        running_state.set(RunningState::RunFailed { msg: format!("Error preparing to run: {}", e) });
                                    },
                                }
                                event.stop_propagation();
                            },
                            img { src: "{format_svg(include_bytes!(\"../../../public/icons/play.svg\"))}" }
                        }
                    }
                    button { class: "img_button {hot_reload_button_class}",
                        onclick: move |event| {
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
                            event.stop_propagation();
                        },
                        p { "hot reload" }
                    }
                }
                div { class: "output_actions",
                    button { class: "img_button",
                        onclick: move |event| {
                            match layer_only_visible().layer {
                                Some(layer) => {
                                    tokio::spawn(async move {
                                        console().info("...sending plot");
                                        let plot_result = send_task(plottery_server_lib::task::Task::Plot {
                                            layer: layer.clone(),
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
                            event.stop_propagation();
                        },
                        p { "send plot" }
                    }
                    button { class: "img_button",
                        onclick: move |event| {
                            if let Some(svg) = svg() {
                                let path = std::env::temp_dir().join("temp_editor.svg");
                                std::fs::write(&path, svg).unwrap();
                                if let Err(err) = open::that_in_background(path).join() {
                                    tracing::error!("Failed to open svg: {:?}", err);
                                }
                            }
                            event.stop_propagation();
                        },
                        p { "open svg" }
                    }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    ParamsEditor {
                        project_params,
                        project_runner,
                        running_state,
                        console,
                        release,
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
                    div { class: "console_container",
                        {
                            let button_text = if console_expanded() { "▼" } else { "▲" };
                            rsx! {
                                button {
                                    class: "console_toggle",
                                    onclick: move |_| {
                                        let current = console_expanded();
                                        console_expanded.set(!current);
                                    },
                                    "{button_text}"
                                }
                            }
                        }
                        div { class: if console_expanded() { "console expanded" } else { "console" },
                            Console {
                                console: console,
                            }
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
