use dioxus::prelude::*;
use dioxus_router::Routable;

use crate::router::browser::browser::Browser;
use crate::router::editor::editor::Editor;
use crate::router::project_add::project_add::ProjectAdd;
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
}
