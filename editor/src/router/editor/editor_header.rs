//! The parts of the project editor's header bar.

use std::path::{Path, PathBuf};

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use plottery_lib::SampleSettings;
use plottery_server_lib::{plot_settings::PlotSettings, send_task, task::Task};
use rfd::AsyncFileDialog;

use crate::{
    components::{
        icon::{Icon, IconName},
        remote::{poke_plotter, RemoteOpen},
        ui::{Button, IconButton, StatusPill, Switch, Tone, Variant},
    },
    model::app_state::{update_app_state, AppState, Panel, PanelLayout},
    router::editor::{
        editor_ctx::EditorCtx,
        editor_data::{use_plot_estimate, LayerChangeWrapper, PreviewSvg},
        external_editor_button::{installed_external_editors, ExternalEditorButton, OpensDir},
        running_state::RunningState,
        variations::{roll_panel::ProgressBar, variations_state::EditorView},
    },
    routes::Route,
    util::format_svg,
};

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

/// Back button, folder crumbs, project name, the view switch and the buttons that open it elsewhere.
#[component]
pub fn ProjectTitle(cargo_path: PathBuf) -> Element {
    let ctx = use_context::<EditorCtx>();
    let project = ctx.project;
    let console = ctx.console;
    let nav = use_navigator();
    let app_state = use_context::<Signal<AppState>>();
    let external_editors = use_hook(installed_external_editors);
    // Crumbs inside a watched folder get a path so they can open the browser there.
    let crumbs = use_memo(move || {
        let state = app_state.peek();
        parent_crumbs(&project().dir)
            .into_iter()
            .map(|(name, path)| (name, state.covers(&path).then_some(path)))
            .collect::<Vec<_>>()
    });
    let cargo_dir = use_memo(move || cargo_path.clone());
    let project_dir = use_memo(move || project().get_dir());
    let icon_folder = format_svg(include_bytes!("../../../public/icons/finder.svg"));

    rsx! {
        IconButton {
            icon: IconName::ChevronLeft,
            label: "Back to projects",
            onclick: move |_| {
                nav.push(Route::Browser {});
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
                            update_app_state(app_state, |state| state.reveal_folder(&path));
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
        ViewSwitch {}
        div { class: "vsep" }
        div { class: "hgroup tight",
            for editor in external_editors.iter().filter(|e| e.opens == OpensDir::Cargo) {
                ExternalEditorButton { editor: *editor, dir: cargo_dir }
            }
            button {
                r#type: "button",
                class: "btn btn-ghost icon-btn app-btn",
                title: "Open project folder",
                "aria-label": "Open project folder",
                onclick: move |event| {
                    if let Err(err) = open::that(project().dir) {
                        console.peek().error(&format!("Failed to open the project folder: {err}"));
                    }
                    event.stop_propagation();
                },
                img { src: "{icon_folder}", alt: "" }
            }
            for editor in external_editors.iter().filter(|e| e.opens == OpensDir::Project) {
                ExternalEditorButton { editor: *editor, dir: project_dir }
            }
        }
    }
}

/// Switches between the Edit and the Variations view.
#[component]
fn ViewSwitch() -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut view = ctx.variations.view;
    let options = [
        (EditorView::Edit, IconName::Sliders, "Edit"),
        (EditorView::Variations, IconName::Dice, "Variations"),
    ];
    rsx! {
        div { class: "view-switch", role: "group", "aria-label": "View",
            for (option, icon, text) in options {
                button {
                    key: "{text}",
                    r#type: "button",
                    class: if view() == option { "on" },
                    "aria-pressed": if view() == option { "true" } else { "false" },
                    title: "{text} (⌘E switches)",
                    onclick: move |_| view.set(option),
                    Icon { name: icon }
                    span { "{text}" }
                }
            }
        }
    }
}

/// Header pill for the run state; nothing while idle.
fn run_status(state: &RunningState) -> Option<(Tone, &'static str)> {
    let status = match state {
        RunningState::Idle => return None,
        RunningState::Preparing { .. }
        | RunningState::StartingBuild { .. }
        | RunningState::Building { .. } => (Tone::Busy, "Compiling…"),
        RunningState::StartingGetParams { .. }
        | RunningState::GetParams { .. }
        | RunningState::StartingRun { .. }
        | RunningState::Running { .. }
        | RunningState::Updating { .. } => (Tone::Busy, "Running…"),
        RunningState::Rolling { .. } => (Tone::Busy, "Rolling…"),
        RunningState::BuildFailed { .. } => (Tone::Err, "Build failed"),
        RunningState::GetParamsFailed { .. } | RunningState::RunFailed { .. } => {
            (Tone::Err, "Run failed")
        }
    };
    Some(status)
}

/// Run button, hot reload switch and the run state.
#[component]
pub fn RunControls(hot_reload: bool, on_hot_reload: Callback<bool>) -> Element {
    let ctx = use_context::<EditorCtx>();
    let running_state = ctx.running_state;
    let state_now = running_state();
    let status = run_status(&state_now);
    let status_title = Some(state_now.get_msg()).filter(|msg| !msg.is_empty());

    let editing = (ctx.variations.view)() == EditorView::Edit;

    rsx! {
        if editing {
            Switch {
                class: "toggle-box",
                label: "Hot reload".to_string(),
                checked: hot_reload,
                onchange: move |enabled| on_hot_reload.call(enabled),
            }
            Button {
                variant: Variant::Primary,
                icon: IconName::Play,
                onclick: move |_| ctx.trigger_run(),
                "Run"
            }
        }
        if let RunningState::Rolling { done, total } = state_now {
            span { class: "pill", role: "status", title: status_title,
                span { class: "dot busy" }
                span { "Rolling" }
                ProgressBar { progress: Some(done as f32 / total.max(1) as f32) }
                span { class: "pill-count", "{done}/{total}" }
            }
        } else if let Some((tone, text)) = status {
            StatusPill { tone, text, title: status_title }
        }
    }
}

/// Buttons that show and hide the side panels and the console.
#[component]
pub fn PanelToggles(layout: PanelLayout, on_toggle: Callback<Panel>) -> Element {
    let shortcut = |key: &str| format!("⌘{key}");
    rsx! {
        IconButton {
            icon: IconName::PanelLeft,
            label: "Toggle parameters",
            title: "Toggle parameters ({shortcut(\"1\")})",
            active: layout.params_open,
            onclick: move |_| on_toggle.call(Panel::Params),
        }
        IconButton {
            icon: IconName::PanelBottom,
            label: "Toggle console",
            title: "Toggle console ({shortcut(\"3\")})",
            active: layout.console_open,
            onclick: move |_| on_toggle.call(Panel::Console),
        }
        IconButton {
            icon: IconName::PanelRight,
            label: "Toggle layers",
            title: "Toggle layers ({shortcut(\"2\")})",
            active: layout.layers_open,
            onclick: move |_| on_toggle.call(Panel::Layers),
        }
    }
}

/// Plotter remote, plot settings, the svg menu, and sending the visible layers.
#[component]
pub fn PlotActions(
    svg: SyncSignal<Option<PreviewSvg>>,
    only_visible: Memo<LayerChangeWrapper>,
    plot_settings: SyncSignal<PlotSettings>,
    on_plot_settings: Callback<()>,
) -> Element {
    let ctx = use_context::<EditorCtx>();
    let console = ctx.console;
    let project = ctx.project;
    let RemoteOpen(mut remote_open) = use_context::<RemoteOpen>();
    let plot_estimate_text = use_plot_estimate(only_visible, plot_settings);
    let mut svg_menu_open = use_signal(|| false);

    rsx! {
        IconButton {
            icon: IconName::Plotter,
            label: "Plotter remote",
            title: "Plotter remote (R)",
            onclick: move |_| remote_open.set(true),
        }
        Button {
            variant: Variant::Ghost,
            icon: IconName::Sliders,
            onclick: move |_| on_plot_settings.call(()),
            "Plot settings"
        }
        div { class: "menu-anchor",
            Button {
                icon: IconName::External,
                onclick: move |_| svg_menu_open.set(true),
                "SVG"
            }
            if svg_menu_open() {
                div {
                    class: "menu-backdrop",
                    onclick: move |_| svg_menu_open.set(false),
                }
                div { class: "menu", role: "menu",
                    button {
                        r#type: "button",
                        class: "menu-item",
                        role: "menuitem",
                        onclick: move |_| {
                            svg_menu_open.set(false);
                            if let Some(svg) = svg() {
                                let path = std::env::temp_dir().join("temp_editor.svg");
                                // A copy, so the other app keeps its file when the preview is rendered again.
                                let opened = std::fs::copy(&svg.path, &path).and_then(|_| open::that(&path));
                                if let Err(err) = opened {
                                    console.peek().error(&format!("Failed to open the SVG: {err}"));
                                }
                            }
                        },
                        "Open"
                    }
                    button {
                        r#type: "button",
                        class: "menu-item",
                        role: "menuitem",
                        onclick: move |_| {
                            svg_menu_open.set(false);
                            async move {
                                let Some(svg) = svg() else {
                                    return;
                                };
                                let file_name = format!("{}.svg", project.peek().config.name);
                                let Some(target) = AsyncFileDialog::new()
                                    .set_title("Save the SVG")
                                    .set_file_name(file_name)
                                    .add_filter("SVG", &["svg"])
                                    .set_parent(&*dioxus::desktop::window().window)
                                    .save_file()
                                    .await
                                    else {
                                    return;
                                };
                                let target = target.path();
                                match std::fs::copy(&svg.path, target) {
                                    Ok(_) => console.peek().success(&format!("Saved the SVG to {}", target.display())),
                                    Err(err) => console.peek().error(&format!("Failed to save the SVG: {err}")),
                                }
                            }
                        },
                        "Save to…"
                    }
                }
            }
        }
        Button {
            variant: Variant::Primary,
            onclick: move |_| {
                match only_visible().layer {
                    Some(layer) => {
                        remote_open.set(true);
                        let current_plot_settings = plot_settings();
                        tokio::spawn(async move {
                            if let Ok(console_read) = console.try_read() {
                                console_read.info("Sending plot…");
                            }

                            let plot_result = send_task(Task::Plot {
                                layer,
                                sample_settings: SampleSettings::default(),
                                plot_settings: current_plot_settings
                            }).await;
                            poke_plotter();
                            if let Ok(console_read) = console.try_read() {
                                match plot_result {
                                    Ok(()) => console_read.success("Plot sent to the plotter"),
                                    Err(err) => console_read.error(format!("Failed to send the plot: {err:#}").as_str()),
                                }
                            }
                        });
                    },
                    None => {
                        console().error("Cannot send the plot: no layer yet");
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
