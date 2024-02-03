use dioxus::prelude::*;
use plottery_project::Project;

use crate::{components::project_overview::ProjectOverview, model::app_state::AppState};

#[derive(Props)]
pub struct ProjectListProps<'a> {
    pub app_state: AppState,
    pub on_delete_clicked: EventHandler<'a, Project>,
}

impl PartialEq for ProjectListProps<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.app_state == other.app_state
    }
}

#[component]
pub fn ProjectList<'a>(cx: Scope<'a, ProjectListProps>) -> Element<'a> {
    cx.render(rsx!(
        style { include_str!("./project_list.css") }
        div { class: "ProjectList",
            cx.props.app_state.projects.iter().map(|project| {
                rsx! {
                    ProjectOverview {
                        project: project.clone(),
                        on_delete_clicked: move |project: Project| {
                            cx.props.on_delete_clicked.call(project);
                        }
                    }
                }
            })
        }
    ))
}
