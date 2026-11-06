use crate::routes::Route;
use crate::util::project_from_segments;
use bincode::{deserialize, serialize};
use dioxus::prelude::*;
use dioxus_logger::tracing;
use dioxus_router::hooks::use_navigator;
use plottery_server_lib::PlotSettings;

use crate::{
    components::navigation::Navigation,
    router::editor::plot_settings_editor::plot_settings_editor::PlotSettingsEditor,
};

#[component]
pub fn ProjectPlotSettings(project_path: Vec<String>) -> Element {
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

    let plot_settings = use_signal_sync(|| {
        let plot_settings_file_path = project().get_editor_plot_settings_path();
        match std::fs::read(plot_settings_file_path) {
            Ok(plot_settings_binary) => deserialize(&plot_settings_binary).unwrap_or_default(),
            Err(_) => PlotSettings::default(),
        }
    });

    use_effect(move || {
        let plot_settings_binary =
            serialize(&(plot_settings())).expect("Failed to serialize plot settings");
        let plot_settings_file_path = project().get_editor_plot_settings_path();
        std::fs::write(plot_settings_file_path, plot_settings_binary)
            .expect("Failed to write plot settings to file");
    });

    rsx! {
        style { { include_str!("./project_plot_settings.css") } }
        Navigation { page_name: "Plot settings", body: rsx! {} }

        div { class: "ProjectPlotSettings",
            div { class: "plot_settings_content",
                PlotSettingsEditor {
                    plot_settings,
                }
            }
        }
    }
}
