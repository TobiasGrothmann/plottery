use crate::router::browser::browser::Browser;
use crate::router::editor::editor::Editor;
use crate::router::project_add::project_add::ProjectAdd;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Routable, PartialEq, Debug, Clone)]
pub enum Route {
    #[route("/")]
    Browser {},
    #[route("/editor/:project_path")]
    Editor { project_path: String },
    #[route("/project_add")]
    ProjectAdd {},
}
