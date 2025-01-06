use crate::router::add_project::ProjectAdd;
use crate::router::{Browser, Editor};
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
