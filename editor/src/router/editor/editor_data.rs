//! Data hooks of the project editor: what is loaded, saved and derived for the panels.

use std::time::Duration;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_lib::{BoundingBox, Layer, Rect, SampleSettings};
use plottery_project::Project;
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;
use plottery_server_lib::{estimation::estimate_plot_layer_duration, plot_settings::PlotSettings};

use crate::{
    model::{
        app_state::{update_app_state, AppState},
        layer_view::{
            filter_visible, LayerExpansion, LayerRows, LayerSummary, LayerViewState,
            LayerVisibility,
        },
    },
    router::editor::{
        console_messages::{ActiveConsole, ConsoleMessages},
        editor_ctx::EditorCtx,
        params_file::ParamsFile,
    },
};

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

/// The console of this editor, registered as the app's active console while mounted.
pub fn use_console() -> (SyncSignal<ConsoleMessages>, SyncSignal<u32>) {
    let change_counter = use_signal_sync(|| 0);
    let console = use_signal_sync(|| ConsoleMessages::new(change_counter));
    let ActiveConsole(mut active_console) = use_context::<ActiveConsole>();
    use_hook(|| active_console.set(Some(console)));
    use_drop(move || {
        if *active_console.peek() == Some(console) {
            active_console.set(None);
        }
    });
    (console, change_counter)
}

pub fn use_project_params(
    project: Signal<Project>,
    console: SyncSignal<ConsoleMessages>,
) -> SyncSignal<ProjectParamsListWrapper> {
    use_signal_sync(|| match project.peek().load_editor_params() {
        Ok((params, skipped)) => {
            if !skipped.is_empty() {
                console.peek().warn(&format!(
                    "Saved params that could not be read get their default: {}",
                    skipped.join(", ")
                ));
            }
            params
        }
        Err(err) => {
            console.peek().error(&format!(
                "{err:#}. Params are not saved until the file is fixed or deleted."
            ));
            ProjectParamsListWrapper::empty()
        }
    })
}

pub fn use_plot_settings(
    project: Signal<Project>,
    console: SyncSignal<ConsoleMessages>,
) -> SyncSignal<PlotSettings> {
    use_signal_sync(|| {
        PlotSettings::load_file(&project.peek().get_editor_plot_settings_path()).unwrap_or_else(
            |err| {
                console.peek().error(&format!(
                    "{err:#}. Plot settings are not saved until the file is fixed or deleted."
                ));
                PlotSettings::default()
            },
        )
    })
}

/// The layer of the last run, read in the background; the bool is true until it is read.
pub fn use_saved_layer(
    project: Signal<Project>,
    console: SyncSignal<ConsoleMessages>,
) -> (SyncSignal<LayerChangeWrapper>, SyncSignal<bool>) {
    let mut layer = use_signal_sync(|| LayerChangeWrapper {
        layer: None,
        change_counter: 0,
    });
    // True until the stored layer is read, or turns out to be missing or unreadable.
    let mut layer_loading = use_signal_sync(|| project.peek().get_editor_layer_path().exists());
    use_effect(move || {
        let layer_path = project().get_editor_layer_path();
        tokio::spawn(async move {
            let read_result =
                tokio::task::spawn_blocking(move || -> Result<Option<Layer>, String> {
                    match std::fs::read(layer_path) {
                        Ok(layer_binary) => Layer::new_from_binary(&layer_binary)
                            .map(Some)
                            .map_err(|err| format!("Failed to load the last layer: {err}")),
                        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
                        Err(err) => Err(format!("Failed to read the last layer: {err}")),
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
                        console_read.error(&format!("Failed to load the last layer: {err}"));
                    }
                }
            }
            if let Ok(mut layer_loading_write) = layer_loading.try_write() {
                *layer_loading_write = false;
            }
        });
    });
    (layer, layer_loading)
}

/// Hidden and collapsed layers of the layers panel, and the layer they leave visible.
#[derive(Clone, Copy)]
pub struct LayerView {
    pub visibility: Signal<LayerVisibility>,
    pub expansion: Signal<LayerExpansion>,
    pub rows: Memo<Option<LayerRows>>,
    pub only_visible: Memo<LayerChangeWrapper>,
    /// Extent of the whole drawing, so the preview stays in place while layers are hidden.
    pub bbox: Memo<Option<Rect>>,
    project: Signal<Project>,
    app_state: Signal<AppState>,
}

impl LayerView {
    /// Hidden and collapsed layers are kept by layer name, so they outlive runs and restarts.
    pub fn save(&self) {
        let dir = self.project.peek().dir.clone();
        let state = LayerViewState {
            visibility: self.visibility.peek().clone(),
            expansion: self.expansion.peek().clone(),
        };
        update_app_state(self.app_state, |app_state| {
            app_state.set_layer_view(&dir, state)
        });
    }
}

pub fn use_layer_view(
    project: Signal<Project>,
    layer: SyncSignal<LayerChangeWrapper>,
) -> LayerView {
    let app_state = use_context::<Signal<AppState>>();
    let saved = use_hook(|| {
        app_state
            .peek()
            .layer_views
            .get(&project.peek().dir)
            .cloned()
            .unwrap_or_default()
    });
    let visibility = use_signal({
        let visibility = saved.visibility.clone();
        move || visibility
    });
    let expansion = use_signal(move || saved.expansion);
    let summary = use_memo(move || layer.read().layer.as_ref().map(LayerSummary::new));
    let rows = use_memo(move || {
        summary
            .read()
            .as_ref()
            .map(|summary| LayerRows::new(summary, &visibility.read(), &expansion.read()))
    });
    let mut only_visible_change_counter = use_signal(|| 0);
    let only_visible = use_memo(move || {
        let mut cc = only_visible_change_counter.write();
        let new_count = *cc + 1;
        *cc = new_count;

        LayerChangeWrapper {
            layer: layer
                .read()
                .layer
                .as_ref()
                .map(|layer| filter_visible(layer, &visibility.read())),
            change_counter: new_count,
        }
    });
    let bbox = use_memo(move || {
        layer
            .read()
            .layer
            .as_ref()
            .and_then(|layer| layer.bounding_box())
    });
    LayerView {
        visibility,
        expansion,
        rows,
        only_visible,
        bbox,
        project,
        app_state,
    }
}

/// The svg of the visible layers, rendered in the background.
pub fn use_preview_svg(layer_view: LayerView) -> SyncSignal<Option<String>> {
    let mut svg = use_signal_sync(|| None as Option<String>);
    use_effect(move || {
        let layer = (layer_view.only_visible)().layer;
        let view = (layer_view.bbox)();

        if let Some(layer) = layer {
            tokio::spawn(async move {
                let render = move || match view {
                    Some(view) => layer.to_svg_in(view).to_string(),
                    None => layer.to_svg().to_string(),
                };
                match tokio::task::spawn_blocking(render).await {
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
    svg
}

/// Formatted estimate of how long plotting the visible layers takes, computed in the background.
pub fn use_plot_estimate(
    only_visible: Memo<LayerChangeWrapper>,
    plot_settings: SyncSignal<PlotSettings>,
) -> Memo<Option<String>> {
    let mut estimated_plot_duration = use_signal_sync(|| None as Option<Duration>);
    use_effect(move || {
        let layer = only_visible().layer;
        let plot_settings = plot_settings();
        estimated_plot_duration.set(None);

        if let Some(layer) = layer {
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
    use_memo(move || {
        estimated_plot_duration()
            .map(|duration| format!("~{}", crate::util::format_plot_duration(duration)))
    })
}

fn save_preview(project: &Project, svg: &str) {
    if let Err(e) = project.save_editor_preview(svg) {
        tracing::error!("Failed to write editor preview: {:?}", e);
    }
}

/// Saves params, plot settings, the layer and the cached preview whenever they change.
pub fn use_autosave(
    ctx: EditorCtx,
    params_file: ParamsFile,
    plot_settings: SyncSignal<PlotSettings>,
    layer: SyncSignal<LayerChangeWrapper>,
    layer_view: LayerView,
    svg: SyncSignal<Option<String>>,
) {
    let EditorCtx {
        project,
        project_params,
        console,
        ..
    } = ctx;
    use_effect(move || {
        if let Err(e) = params_file.save(&project_params()) {
            console
                .peek()
                .error(&format!("Failed to save params: {e:#}"));
        }
    });
    use_effect(move || {
        let path = project().get_editor_plot_settings_path();
        if let Err(e) = plot_settings().save_file(&path) {
            console
                .peek()
                .error(&format!("Failed to save plot settings: {e:#}"));
        }
    });
    use_effect(move || {
        if let Some(layer) = layer.read().layer.as_ref() {
            if let Err(e) = project().save_editor_layer(layer) {
                console
                    .peek()
                    .error(&format!("Failed to save the layer: {e:#}"));
            }
        }
    });
    // The cached preview and the thumbnail show the whole drawing: the svg on screen while
    // nothing is hidden, otherwise one more render of each new layer.
    let visibility = layer_view.visibility;
    use_effect(move || {
        if let Some(svg) = svg().filter(|_| !visibility.peek().hides_anything()) {
            let project = project();
            tokio::task::spawn_blocking(move || save_preview(&project, &svg));
        }
    });
    use_effect(move || {
        let whole_layer = layer.read().layer.clone();
        if let Some(layer) = whole_layer.filter(|_| visibility.peek().hides_anything()) {
            let project = project();
            tokio::task::spawn_blocking(move || {
                save_preview(&project, &layer.to_svg().to_string())
            });
        }
    });
}
