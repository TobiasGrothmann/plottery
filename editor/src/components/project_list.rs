use dioxus::prelude::*;

use crate::{components::project_overview::ProjectOverview, model::app_state::AppState};

#[derive(PartialEq, Props)]
pub struct ProjectListProps {
    pub app_state: AppState,
}

#[component]
pub fn ProjectList(cx: Scope<ProjectListProps>) -> Element {
    cx.render(rsx!(
        style { include_str!("./project_list.css") }
        div { class: "ProjectList",
            h1 { "Projects" }
            main {
                div { class: "ProjectList",
                    cx.props.app_state.projects.iter().map(|project| {
                        rsx! {
                            ProjectOverview {
                                project: project.clone(),
                            }
                        }
                    })
                }
            }
        }
    ))
}
