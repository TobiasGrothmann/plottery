use bincode::{deserialize, serialize};
use dioxus::prelude::*;
use plottery_project::Project;
use plottery_server_lib::PlotSettings;
use std::path::PathBuf;

use crate::{
    components::navigation::Navigation,
    router::editor::plot_settings_editor::plot_settings_editor::PlotSettingsEditor,
};

#[component]
pub fn ProjectPlotSettings(project_path: Vec<String>) -> Element {
    let project = use_signal(|| {
        let path_str = format!("/{}", project_path.join("/"));
        let p = PathBuf::from(path_str);
        Project::load_from_file(p).expect("Failed to load project from file")
    });

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
