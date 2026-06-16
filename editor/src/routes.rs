use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::router::browser::browser::Browser;
use crate::router::editor::editor::Editor;
use crate::router::project_add::project_add::ProjectAdd;
use crate::router::project_plot_settings::project_plot_settings::ProjectPlotSettings;
use crate::router::remote::remote::Remote;

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route {
    #[route("/")]
    Browser {},
    #[route("/editor/:..project_path")]
    Editor { project_path: Vec<String> },
    #[route("/project_add")]
    ProjectAdd {},
    #[route("/remote")]
    Remote {},
    #[route("/project_plot_settings/:..project_path")]
    ProjectPlotSettings { project_path: Vec<String> },
}
