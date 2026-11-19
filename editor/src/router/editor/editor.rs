use crate::util::project_from_segments;
use crate::{
    components::{
        icon::IconName,
        modal::{Modal, ModalWidth},
        resizer::{Axis, Resizer},
        ui::{Button, IconButton, PanelHeader, Size, Variant},
    },
    model::{
        app_state::{update_app_state, AppState, Panel},
        layer_view::{LayerKey, RowTarget},
    },
    router::editor::{
        console::Console,
        editor_ctx::EditorCtx,
        editor_data::{
            use_autosave, use_console, use_layer_view, use_plot_settings, use_preview_svg,
            use_project_params, use_saved_layer,
        },
        editor_header::{PanelToggles, PlotActions, ProjectTitle, RunControls},
        layer_editor::layer_editor::LayerEditor,
        params_editor::params_editor::ParamsEditor,
        params_file::ParamsFile,
        pick_target::PickTarget,
        plot_settings_editor::plot_settings_editor::PlotSettingsEditor,
        preview_stage::PreviewStage,
        project_hot_reload::use_hot_reload,
        project_runner::ProjectRunner,
        running_state::RunningState,
    },
    routes::Route,
    util::preview_zoom::PreviewZoom,
};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use dioxus_router::hooks::use_navigator;
use plottery_server_lib::plot_settings::PlotSettings;

/// Eval script: true while a text field has focus, so single-key shortcuts stay out of the way.
const FOCUS_IS_TEXT_FIELD: &str = r#"
const el = document.activeElement;
return !!el && (["INPUT", "TEXTAREA", "SELECT"].includes(el.tagName) || el.isContentEditable);
"#;

/// Keyed by project, so switching projects remounts instead of keeping the old one loaded.
#[component]
pub fn Editor(project_path: Vec<String>) -> Element {
    let key = project_path.join("/");
    // dioxus only honors keys in lists, so a one-item list remounts on a new project.
    rsx! {
        for key in std::iter::once(key) {
            ProjectEditor { key: "{key}", project_path: project_path.clone() }
        }
    }
}

#[component]
fn ProjectEditor(project_path: Vec<String>) -> Element {
    let loaded = use_hook(|| {
        project_from_segments(&project_path)
            .and_then(|project| Ok((project.get_cargo_path()?, project)))
            .map_err(|e| e.to_string())
    });
    let nav = use_navigator();
    let (cargo_path, loaded) = match loaded {
        Ok(loaded) => loaded,
        Err(err) => {
            tracing::error!("Cannot open project, returning home: {err}");
            nav.replace(Route::Browser {});
            return rsx! {};
        }
    };
    let project = use_signal(move || loaded);
    let release = true;

    // data
    let (console, console_change_counter) = use_console();
    let project_params = use_project_params(project, console);
    let plot_settings = use_plot_settings(project, console);
    let (layer, layer_loading) = use_saved_layer(project, console);
    let layer_view = use_layer_view(project, layer);
    let svg = use_preview_svg(layer_view);
    let params_file = use_hook(|| ParamsFile::new(project.peek().clone()));
    // The cached preview is shown until the layer is loaded and rendered. Checked once: loading
    // the layer saves it again, which makes the cache look outdated until the new preview is saved.
    let cached_preview = use_hook(|| {
        project
            .peek()
            .fresh_editor_preview()
            .map(|path| path.to_string_lossy().to_string())
    });

    // running project
    let running_state = use_signal_sync(|| RunningState::Idle);
    let project_runner =
        use_signal_sync(|| ProjectRunner::new(project.peek().clone(), layer, project_params));
    let ctx = use_context_provider(|| EditorCtx {
        project,
        project_params,
        project_runner,
        running_state,
        console,
        release,
    });
    use_context_provider(|| PickTarget {
        path: Signal::new(None),
    });
    use_autosave(
        ctx,
        params_file.clone(),
        plot_settings,
        layer,
        layer_view,
        svg,
    );
    let (hot_reload, set_hot_reload) = use_hot_reload(ctx, cargo_path.clone(), params_file);

    let mut preview_zoom = use_signal(PreviewZoom::default);
    let mut fullscreen = use_signal(|| false);
    let mut show_plot_settings = use_signal(|| false);
    let mut plot_settings_for_reset = plot_settings;
    let mut layer_visibility = layer_view.visibility;
    let mut layer_expansion = layer_view.expansion;

    // panels
    let app_state = use_context::<Signal<AppState>>();
    let mut layout = use_signal(|| app_state.peek().panel_layout.clone().clamped());
    let save_layout = move || {
        update_app_state(app_state, |state| {
            state.panel_layout = layout.peek().clone()
        });
    };
    let mut toggle_panel = move |panel: Panel| {
        layout.write().toggle(panel);
        save_layout();
    };
    // Opens the console when a new error is logged.
    let mut seen_errors = use_signal(|| 0usize);
    use_effect(move || {
        console_change_counter();
        let errors = console.peek().error_count();
        let seen = *seen_errors.peek();
        if errors != seen {
            seen_errors.set(errors);
        }
        if errors > seen && !layout.peek().console_open {
            toggle_panel(Panel::Console);
        }
    });
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
    let (params_min, params_max, params_default) = Panel::Params.size_limits();
    let (layers_min, layers_max, layers_default) = Panel::Layers.size_limits();
    let (console_min, console_max, console_default) = Panel::Console.size_limits();

    let busy = running_state().is_busy();
    let has_drawing = layer_loading() || layer.read().layer.is_some();

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
                let f = matches!(event.key(), Key::Character(ref key) if key.eq_ignore_ascii_case("f"));
                if f && !command && !modifiers.alt() {
                    spawn(async move {
                        let typing = document::eval(FOCUS_IS_TEXT_FIELD).await;
                        if typing.ok().and_then(|v| v.as_bool()) != Some(true) {
                            fullscreen.toggle();
                            preview_zoom.set(PreviewZoom::default());
                        }
                    });
                    return;
                }
                if event.key() == Key::Escape && fullscreen() {
                    event.prevent_default();
                    fullscreen.set(false);
                    preview_zoom.set(PreviewZoom::default());
                    return;
                }
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
                    ProjectTitle { cargo_path }
                }
                div { class: "hgroup center-group",
                    RunControls { hot_reload, on_hot_reload: set_hot_reload }
                }
                div { class: "hgroup end",
                    div { class: "hgroup tight",
                        PanelToggles {
                            layout: current_layout.clone(),
                            on_toggle: toggle_panel,
                        }
                    }
                    div { class: "vsep" }
                    PlotActions {
                        svg,
                        only_visible: layer_view.only_visible,
                        plot_settings,
                        on_plot_settings: move |_| show_plot_settings.set(true),
                    }
                }
            }

            div { class: "workspace",
                if current_layout.params_open {
                    aside { class: "params-panel", style: "width: {current_layout.params_w}px;",
                        PanelHeader {
                            title: "Parameters",
                            on_collapse: move |_| toggle_panel(Panel::Params),
                        }
                        div { class: "panel-body scroll",
                            ParamsEditor {}
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
                    PreviewStage {
                        svg,
                        cached_preview: cached_preview.clone(),
                        layer_bbox: (layer_view.bbox)(),
                        redraw_counter: layer().change_counter,
                        busy,
                        has_drawing,
                        fullscreen,
                        zoom: preview_zoom,
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
                                rows: layer_view.rows,
                                on_toggle_visible: move |target: RowTarget| {
                                    layer_visibility.write().toggle(&target);
                                    layer_view.save();
                                },
                                on_solo: move |target: RowTarget| {
                                    layer_visibility.write().toggle_solo(target)
                                },
                                on_set_expanded: move |(key, expanded): (LayerKey, bool)| {
                                    layer_expansion.write().set(key, expanded);
                                    layer_view.save();
                                },
                                on_show_all: move |_| {
                                    layer_visibility.write().show_all();
                                    layer_view.save();
                                },
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
