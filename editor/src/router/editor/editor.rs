use crate::util::project_from_segments;
use crate::{
    components::{
        icon::IconName,
        image::*,
        loading_spinner::Loading,
        modal::{Modal, ModalWidth},
        resizer::{Axis, Resizer},
        ui::{Button, IconButton, PanelHeader, Size, StatusPill, Switch, Tone, Variant},
    },
    model::app_state::{AppState, Panel},
    router::editor::{
        console::Console,
        console_messages::ConsoleMessages,
        external_editor_button::ExternalEditorButton,
        layer_editor::{layer_editor::LayerEditor, layer_tree_ref::LayerTreeReference},
        params_editor::{
            param_tree::{get_param_value_mut_by_path, leaf_value_mut},
            params_editor::ParamsEditor,
        },
        pick_target::PickTarget,
        plot_settings_editor::plot_settings_editor::PlotSettingsEditor,
        project_hot_reload::start_hot_reload,
        project_runner::ProjectRunner,
        running_state::RunningState,
    },
    routes::Route,
    util::{format_svg, preview_zoom::PreviewZoom},
};
use bincode::{deserialize, serialize};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use dioxus_router::hooks::use_navigator;
use notify::FsEventWatcher;
use plottery_lib::{BoundingBox, Layer, LayerPropsInheritable, SampleSettings, V2};
use plottery_project::{
    project_param_value::ProjectParamValue, project_params_list_wrapper::ProjectParamsListWrapper,
};
use plottery_server_lib::{
    estimation::estimate_plot_layer_duration, plot_setting::PlotSettings, task::send_task,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
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

fn format_estimated_plot_duration(duration: Duration) -> String {
    if duration.is_zero() {
        return "~0m".to_string();
    }

    let total_minutes = (duration.as_secs_f64() / 60.0).ceil() as u64;

    if total_minutes < 60 {
        format!("~{}m", total_minutes.max(1))
    } else {
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;

        if minutes == 0 {
            format!("~{}h", hours)
        } else {
            format!("~{}h {:02}m", hours, minutes)
        }
    }
}

fn run_status(state: &RunningState) -> (Tone, &'static str) {
    match state {
        RunningState::Idle => (Tone::Ok, "Up to date"),
        RunningState::Preparing { .. }
        | RunningState::StartingBuild { .. }
        | RunningState::Building { .. } => (Tone::Busy, "Compiling…"),
        RunningState::StartingGetParams { .. }
        | RunningState::GetParams { .. }
        | RunningState::StartingRun { .. }
        | RunningState::Running { .. }
        | RunningState::Updating { .. } => (Tone::Busy, "Running…"),
        RunningState::BuildFailed { .. } => (Tone::Err, "Build failed"),
        RunningState::GetParamsFailed { .. } | RunningState::RunFailed { .. } => {
            (Tone::Err, "Run failed")
        }
        RunningState::BuildKilled { .. }
        | RunningState::GetParamsKilled { .. }
        | RunningState::RunKilled { .. } => (Tone::Warn, "Cancelled"),
    }
}

/// The last two parent folders of `dir`, outermost first, with their paths.
fn parent_crumbs(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut crumbs: Vec<(String, PathBuf)> = dir
        .ancestors()
        .skip(1)
        .filter_map(|path| {
            Some((
                path.file_name()?.to_string_lossy().to_string(),
                path.to_path_buf(),
            ))
        })
        .take(2)
        .collect();
    crumbs.reverse();
    crumbs
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
    let loaded = use_hook(|| project_from_segments(&project_path).map_err(|e| e.to_string()));
    let nav = use_navigator();
    let Ok(loaded) = loaded else {
        tracing::error!(
            "Cannot open project, returning home: {}",
            loaded.unwrap_err()
        );
        nav.replace(Route::Browser {});
        return rsx! {};
    };
    let project = use_signal(move || loaded);

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
    let plot_settings = use_signal_sync(|| {
        let plot_settings_file_path = project().get_editor_plot_settings_path();
        match std::fs::read(plot_settings_file_path) {
            Ok(plot_settings_binary) => match deserialize(&plot_settings_binary) {
                Ok(plot_settings) => plot_settings,
                Err(err) => {
                    console().error(
                        format!("failed to deserialize plot settings from binary: {:?}", err)
                            .as_str(),
                    );
                    PlotSettings::default()
                }
            },
            Err(_) => PlotSettings::default(),
        }
    });
    let layer = use_signal_sync(|| LayerChangeWrapper {
        layer: None,
        change_counter: 0,
    });
    {
        let mut layer = layer;
        use_effect(move || {
            let layer_path = project().get_editor_layer_path();
            tokio::spawn(async move {
                let read_result =
                    tokio::task::spawn_blocking(move || -> Result<Option<Layer>, String> {
                        match std::fs::read(layer_path) {
                            Ok(layer_binary) => Layer::new_from_binary(&layer_binary)
                                .map(Some)
                                .map_err(|err| {
                                    format!("failed to deserialize layer from binary: {:?}", err)
                                }),
                            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
                            Err(err) => Err(format!("failed to read layer from file: {:?}", err)),
                        }
                    })
                    .await;

                match read_result {
                    Ok(Ok(layer_from_file)) => {
                        let change_counter = match layer.try_read() {
                            Ok(layer_read) => layer_read.change_counter,
                            Err(err) => {
                                tracing::debug!("skipping layer update after drop: {:?}", err);
                                return;
                            }
                        };

                        if let Ok(mut layer_write) = layer.try_write() {
                            *layer_write = LayerChangeWrapper {
                                layer: layer_from_file,
                                change_counter: change_counter + 1,
                            };
                        }
                    }
                    Ok(Err(err_msg)) => {
                        if let Ok(console_read) = console.try_read() {
                            console_read.error(err_msg.as_str());
                        }
                    }
                    Err(err) => {
                        if let Ok(console_read) = console.try_read() {
                            console_read.error(
                                format!("failed to load layer in background: {:?}", err).as_str(),
                            );
                        }
                    }
                }
            });
        });
    }

    let mut layer_tree_ref = use_signal_sync(move || None);
    use_effect(move || {
        let layer_tree = layer
            .read()
            .layer
            .as_ref()
            .map(|layer| LayerTreeReference::new(layer, &LayerPropsInheritable::default()));
        layer_tree_ref.set(layer_tree)
    });

    let mut layer_only_visible_change_counter = use_signal(|| 0);
    let layer_only_visible = use_memo(move || {
        let mut cc = layer_only_visible_change_counter.write();
        let new_count = *cc + 1;
        *cc = new_count;

        let layer_read = layer.read();
        match (layer_tree_ref(), layer_read.layer.as_ref()) {
            (Some(layer_tree_ref), Some(layer)) => LayerChangeWrapper {
                layer: Some(layer_tree_ref.filter_layer_by_visibility(layer)),
                change_counter: new_count,
            },
            _ => LayerChangeWrapper {
                layer: None,
                change_counter: new_count,
            },
        }
    });

    let mut estimated_plot_duration = use_signal_sync(|| None as Option<Duration>);
    use_effect(move || {
        let layer = layer_only_visible().layer;
        let plot_settings = plot_settings();
        estimated_plot_duration.set(None);

        if let Some(layer) = layer {
            let mut estimated_plot_duration = estimated_plot_duration;
            tokio::spawn(async move {
                match tokio::task::spawn_blocking(move || {
                    estimate_plot_layer_duration(&layer, SampleSettings::low_res(), &plot_settings)
                })
                .await
                {
                    Ok(duration) => {
                        if let Ok(mut estimated_plot_duration_write) =
                            estimated_plot_duration.try_write()
                        {
                            *estimated_plot_duration_write = Some(duration);
                        }
                    }
                    Err(err) => {
                        tracing::error!("failed to estimate plot duration: {:?}", err);
                    }
                }
            });
        }
    });

    let plot_estimate_text =
        use_memo(move || estimated_plot_duration().map(format_estimated_plot_duration));

    // hooks for changes in project
    // params
    use_effect(move || {
        let params_binary =
            serialize(&(project_params())).expect("Failed to serialize project params");
        let params_file_path = project().get_editor_params_path();
        std::fs::write(params_file_path, params_binary)
            .expect("Failed to write project params to file");
    });
    // plot settings
    use_effect(move || {
        let plot_settings_binary =
            serialize(&(plot_settings())).expect("Failed to serialize plot settings");
        let plot_settings_file_path = project().get_editor_plot_settings_path();
        std::fs::write(plot_settings_file_path, plot_settings_binary)
            .expect("Failed to write plot settings to file");
    });
    // layer
    let svg = use_signal_sync(|| None as Option<String>);
    use_effect(move || {
        let layer = layer_only_visible().layer;

        if let Some(layer) = layer {
            let mut svg = svg;
            tokio::spawn(async move {
                match tokio::task::spawn_blocking(move || layer.to_svg(1.0).to_string()).await {
                    Ok(svg_str) => {
                        if let Ok(mut svg_write) = svg.try_write() {
                            *svg_write = Some(svg_str);
                        }
                    }
                    Err(err) => tracing::error!("failed to generate preview svg: {:?}", err),
                }
            });
        }
    });
    use_effect(move || {
        let layer_path = project().get_editor_layer_path();
        if let Some(layer) = layer.read().layer.as_ref() {
            let binary = layer.to_binary().expect("Failed to serialize layer");
            match std::fs::write(layer_path, binary) {
                Ok(_) => (),
                Err(e) => tracing::error!("Failed to write layer to file: {:?}", e),
            }
        }

        let svg_path = project().get_editor_preview_image_path();
        if let Some(svg) = svg() {
            match std::fs::write(svg_path, &svg) {
                Ok(_) => (),
                Err(e) => tracing::error!("Failed to write .svg to file: {:?}", e),
            }
            let project = project();
            tokio::task::spawn_blocking(move || {
                if let Err(e) = project.write_editor_thumbnail(svg.as_bytes()) {
                    tracing::error!("Failed to write thumbnail: {:?}", e);
                }
            });
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
    let mut trigger_run = move || {
        running_state.set(RunningState::Preparing {
            msg: "preparing".to_string(),
        });
        match project_runner().try_lock() {
            Ok(mut runner) => runner.trigger_run_project(release, running_state, console),
            Err(e) => {
                tracing::error!("Error preparing to run: {:?}", e);
                running_state.set(RunningState::RunFailed {
                    msg: format!("Error preparing to run: {}", e),
                });
            }
        }
    };

    let mut preview_zoom = use_signal(PreviewZoom::default);

    // click-to-pick for V2 params
    let mut pick_target = use_context_provider(|| PickTarget {
        path: Signal::new(None),
    });
    let layer_bbox = use_memo(move || {
        layer_only_visible()
            .layer
            .as_ref()
            .and_then(|layer| layer.bounding_box())
    });
    let mut project_params_for_pick = project_params;
    let onpick = move |point: V2| {
        let Some(path) = pick_target.take() else {
            return;
        };
        let mut new_params = project_params_for_pick.read().clone();
        match get_param_value_mut_by_path(&mut new_params.list, &path) {
            Some(value) => match leaf_value_mut(value) {
                ProjectParamValue::V2(v) => *v = point,
                _ => {
                    tracing::error!("Pick target is not a V2 param: {:?}", path);
                    return;
                }
            },
            None => {
                tracing::error!("Pick target not found: {:?}", path);
                return;
            }
        }
        project_params_for_pick.set(new_params);
        trigger_run();
    };

    // hot reload
    let mut hot_reload_watcher = use_signal_sync(|| None as Option<FsEventWatcher>);
    let mut hot_reload_join_handle = use_signal_sync(|| None as Option<JoinHandle<()>>);
    let hot_reload_path_to_watch = project().get_cargo_path().unwrap().join("src");
    let hot_reload_is_enabled = move || hot_reload_watcher.read().is_some();

    let mut last_run = use_signal(|| None::<String>);
    let mut seen_busy = use_signal(|| false);
    use_effect(move || {
        let state = running_state();
        if state.is_busy() {
            if !*seen_busy.peek() {
                seen_busy.set(true);
            }
        } else if matches!(state, RunningState::Idle) && *seen_busy.peek() {
            last_run.set(Some(chrono::Local::now().format("%H:%M").to_string()));
        }
    });

    let hot_reload_enabled = hot_reload_is_enabled();
    let set_hot_reload = move |enabled: bool| {
        if enabled == hot_reload_is_enabled() {
            return;
        }
        if enabled {
            let (handle, watcher) = start_hot_reload(
                hot_reload_path_to_watch.clone(),
                release,
                project_runner().clone(),
                running_state,
                console,
            );
            hot_reload_join_handle.set(Some(handle));
            hot_reload_watcher.set(Some(watcher));
            trigger_run();
        } else {
            hot_reload_watcher.set(None);
            hot_reload_join_handle.set(None);
        }
    };

    let state_now = running_state();
    let (status_tone, status_text) = run_status(&state_now);
    let status_detail = last_run().filter(|_| matches!(state_now, RunningState::Idle));
    let status_title = Some(state_now.get_msg()).filter(|msg| !msg.is_empty());
    let busy = state_now.is_busy();
    // Crumbs inside a watched folder get a path so they can open the browser there.
    let crumbs = use_memo(move || {
        let state = AppState::load_or_default();
        parent_crumbs(&project().dir)
            .into_iter()
            .map(|(name, path)| (name, state.covers(&path).then_some(path)))
            .collect::<Vec<_>>()
    });
    let param_count = project_params.read().list.len();

    let mut show_plot_settings = use_signal(|| false);
    let mut plot_settings_for_reset = plot_settings;

    let mut layout = use_signal(|| AppState::load_or_default().panel_layout.clamped());
    let save_layout = move || {
        let mut state = AppState::load_or_default();
        state.panel_layout = layout.peek().clone();
        state.save();
    };
    let mut toggle_panel = move |panel: Panel| {
        layout.write().toggle(panel);
        save_layout();
    };
    let resize_panel = move |panel: Panel| {
        EventHandler::new(move |size: f64| layout.write().set_size(panel, size))
    };
    let commit_panel = move |panel: Panel| {
        EventHandler::new(move |size: f64| {
            layout.write().set_size(panel, size);
            save_layout();
        })
    };
    let current_layout = layout();
    let shortcut = |key: &str| {
        if cfg!(target_os = "macos") {
            format!("⌘{key}")
        } else {
            format!("Ctrl+{key}")
        }
    };
    let (params_min, params_max, params_default) = Panel::Params.size_limits();
    let (layers_min, layers_max, layers_default) = Panel::Layers.size_limits();
    let (console_min, console_max, console_default) = Panel::Console.size_limits();

    let icon_folder = if cfg!(target_os = "windows") {
        format_svg(include_bytes!("../../../public/icons/explorer.svg"))
    } else if cfg!(target_os = "macos") {
        format_svg(include_bytes!("../../../public/icons/finder.svg"))
    } else {
        format_svg(include_bytes!("../../../public/icons/linux_folder.svg"))
    };

    rsx! {
        style { { include_str!("./editor.css") } }
        div {
            class: "Editor",
            tabindex: "-1",
            onmounted: move |event| async move {
                let _ = event.data().set_focus(true).await;
            },
            onkeydown: move |event| {
                let modifiers = event.modifiers();
                let command = modifiers.meta() || modifiers.ctrl();
                let zero = matches!(event.key(), Key::Character(ref key) if key == "0");
                if event.key() == Key::Escape || (command && zero) {
                    if !preview_zoom.peek().is_fit() {
                        event.prevent_default();
                        preview_zoom.set(PreviewZoom::default());
                    }
                    return;
                }
                if !command {
                    return;
                }
                let Key::Character(key) = event.key() else { return };
                let panel = match key.to_lowercase().as_str() {
                    "1" => Panel::Params,
                    "j" => Panel::Console,
                    "2" => Panel::Layers,
                    _ => return,
                };
                event.prevent_default();
                toggle_panel(panel);
            },
            header { class: "app-header",
                div { class: "hgroup",
                    IconButton {
                        icon: IconName::ChevronLeft,
                        label: "Back to projects",
                        onclick: move |_| {
                            if nav.can_go_back() {
                                nav.go_back();
                            } else {
                                nav.replace(Route::Browser {});
                            }
                        },
                    }
                    div { class: "title-block", title: "{project().dir.to_string_lossy()}",
                        for (name, path) in crumbs() {
                            if let Some(path) = path {
                                button {
                                    r#type: "button",
                                    class: "crumb crumb-link",
                                    title: "Show {path.to_string_lossy()} in projects",
                                    onclick: move |_| {
                                        let mut state = AppState::load_or_default();
                                        state.reveal_folder(&path);
                                        state.save();
                                        nav.push(Route::Browser {});
                                    },
                                    "{name}"
                                }
                            } else {
                                span { class: "crumb", "{name}" }
                            }
                            span { class: "crumb", "/" }
                        }
                        h1 { "{project().config.name}" }
                    }
                    div { class: "vsep" }
                    div { class: "hgroup tight",
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
                                button {
                                    r#type: "button",
                                    class: "btn btn-ghost icon-btn app-btn",
                                    title: "Open project folder",
                                    "aria-label": "Open project folder",
                                    onclick: move |event| {
                                        open::that_in_background(project().dir).join().unwrap().unwrap();
                                        event.stop_propagation();
                                    },
                                    img { src: "{icon_folder}", alt: "" }
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
                }
                div { class: "hgroup center-group",
                    Button {
                        variant: Variant::Primary,
                        icon: IconName::Play,
                        onclick: move |_| trigger_run(),
                        "Run"
                    }
                    Switch {
                        class: "toggle-box",
                        label: "Hot reload".to_string(),
                        checked: hot_reload_enabled,
                        onchange: set_hot_reload,
                    }
                    StatusPill {
                        tone: status_tone,
                        text: status_text.to_string(),
                        detail: status_detail,
                        title: status_title,
                    }
                }
                div { class: "hgroup end",
                    div { class: "hgroup tight",
                        IconButton {
                            icon: IconName::PanelLeft,
                            label: "Toggle parameters",
                            title: "Toggle parameters ({shortcut(\"1\")})",
                            active: current_layout.params_open,
                            onclick: move |_| toggle_panel(Panel::Params),
                        }
                        IconButton {
                            icon: IconName::PanelBottom,
                            label: "Toggle console",
                            title: "Toggle console ({shortcut(\"J\")})",
                            active: current_layout.console_open,
                            onclick: move |_| toggle_panel(Panel::Console),
                        }
                        IconButton {
                            icon: IconName::PanelRight,
                            label: "Toggle layers",
                            title: "Toggle layers ({shortcut(\"2\")})",
                            active: current_layout.layers_open,
                            onclick: move |_| toggle_panel(Panel::Layers),
                        }
                    }
                    div { class: "vsep" }
                    Button {
                        variant: Variant::Ghost,
                        icon: IconName::Sliders,
                        onclick: move |_| show_plot_settings.set(true),
                        "Plot settings"
                    }
                    Button {
                        icon: IconName::External,
                        onclick: move |_| {
                            if let Some(svg) = svg() {
                                let path = std::env::temp_dir().join("temp_editor.svg");
                                std::fs::write(&path, svg).unwrap();
                                if let Err(err) = open::that_in_background(path).join() {
                                    tracing::error!("Failed to open svg: {:?}", err);
                                }
                            }
                        },
                        "Open SVG"
                    }
                    Button {
                        variant: Variant::Primary,
                        onclick: move |_| {
                            match layer_only_visible().layer {
                                Some(layer) => {
                                    let current_plot_settings = plot_settings();
                                    tokio::spawn(async move {
                                        if let Ok(console_read) = console.try_read() {
                                            console_read.info("...sending plot");
                                        }

                                        let plot_result = send_task(plottery_server_lib::task::Task::Plot {
                                            layer: layer.clone(),
                                            sample_settings: SampleSettings::default(),
                                            plot_settings: current_plot_settings
                                        }).await;
                                        if let Err(err) = plot_result {
                                            if let Ok(console_read) = console.try_read() {
                                                console_read.error(format!("failed to send plot: {:?}", err).as_str());
                                            }
                                        }
                                    });
                                },
                                None => {
                                    console().error("cannot send plot: no layer available");
                                }
                            }
                        },
                        "Send to plotter"
                        if let Some(estimate) = plot_estimate_text() {
                            span { class: "mono estimate", "{estimate}" }
                        }
                    }
                }
            }

            div { class: "workspace",
                if current_layout.params_open {
                    aside { class: "params-panel", style: "width: {current_layout.params_w}px;",
                        PanelHeader {
                            title: "Parameters",
                            count: param_count,
                            on_collapse: move |_| toggle_panel(Panel::Params),
                        }
                        div { class: "panel-body scroll",
                            ParamsEditor {
                                project_dir: project().get_dir(),
                                project_params,
                                project_runner,
                                running_state,
                                console,
                                release,
                            }
                        }
                    }
                    Resizer {
                        axis: Axis::Vertical,
                        value: current_layout.params_w,
                        min: params_min,
                        max: params_max,
                        default: params_default,
                        onchange: resize_panel(Panel::Params),
                        oncommit: commit_panel(Panel::Params),
                        label: "Resize parameters",
                    }
                } else {
                    div { class: "rail rail-left",
                        IconButton {
                            icon: IconName::ChevronRight,
                            label: "Show parameters",
                            size: Size::Sm,
                            onclick: move |_| toggle_panel(Panel::Params),
                        }
                        span { class: "rail-label", "Parameters" }
                    }
                }
                main { class: "center",
                    section { class: if busy { "preview busy" } else { "preview" },
                        if project().get_editor_preview_image_path().exists() {
                            if let Some(svg) = svg() {
                                SVGImage {
                                    svg,
                                    layer_bbox: layer_bbox(),
                                    pick_active: pick_target.is_active(),
                                    onpick,
                                    zoom: preview_zoom,
                                }
                            } else {
                                div { class: "preview-stage",
                                    Image {
                                        class: "paper paper-file",
                                        img_path: project().get_editor_preview_image_path().to_str().unwrap().to_string(),
                                        redraw_counter: layer().change_counter,
                                    }
                                }
                            }
                            if busy {
                                Loading {}
                            }
                        } else {
                            div { class: "preview-empty",
                                p { "No preview yet" }
                                Button {
                                    variant: Variant::Primary,
                                    icon: IconName::Play,
                                    onclick: move |_| trigger_run(),
                                    "Run"
                                }
                            }
                            if busy {
                                Loading {}
                            }
                        }
                    }
                    if current_layout.console_open {
                        Resizer {
                            axis: Axis::Horizontal,
                            value: current_layout.console_h,
                            min: console_min,
                            max: console_max,
                            default: console_default,
                            invert: true,
                            onchange: resize_panel(Panel::Console),
                            oncommit: commit_panel(Panel::Console),
                            label: "Resize console",
                        }
                    }
                    Console {
                        console,
                        open: current_layout.console_open,
                        height: current_layout.console_h,
                        on_toggle: move |_| toggle_panel(Panel::Console),
                    }
                }
                if current_layout.layers_open {
                    Resizer {
                        axis: Axis::Vertical,
                        value: current_layout.layers_w,
                        min: layers_min,
                        max: layers_max,
                        default: layers_default,
                        invert: true,
                        onchange: resize_panel(Panel::Layers),
                        oncommit: commit_panel(Panel::Layers),
                        label: "Resize layers",
                    }
                    aside { class: "layers-panel", style: "width: {current_layout.layers_w}px;",
                        PanelHeader {
                            title: "Layers",
                            collapse_icon: IconName::ChevronRight,
                            on_collapse: move |_| toggle_panel(Panel::Layers),
                        }
                        div { class: "panel-body scroll",
                            LayerEditor {
                                layer_tree_ref,
                                running_state: running_state,
                            }
                        }
                    }
                } else {
                    div { class: "rail rail-right",
                        IconButton {
                            icon: IconName::ChevronLeft,
                            label: "Show layers",
                            size: Size::Sm,
                            onclick: move |_| toggle_panel(Panel::Layers),
                        }
                        span { class: "rail-label", "Layers" }
                    }
                }
            }
            Modal {
                open: show_plot_settings,
                width: ModalWidth::Table,
                title: "Plot settings",
                subtitle: "Saved with {project().config.name}. Changes apply to the next plot you send.",
                body: rsx! {
                    PlotSettingsEditor { plot_settings }
                },
                footer: rsx! {
                    Button {
                        variant: Variant::Ghost,
                        size: Size::Sm,
                        icon: IconName::Reset,
                        onclick: move |_| plot_settings_for_reset.set(PlotSettings::default()),
                        "Reset to defaults"
                    }
                    Button {
                        variant: Variant::Primary,
                        onclick: move |_| show_plot_settings.set(false),
                        "Done"
                    }
                },
            }
        }
    }
}
