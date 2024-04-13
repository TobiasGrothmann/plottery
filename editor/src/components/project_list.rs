use dioxus::prelude::*;
use plottery_project::Project;

use crate::{components::project_overview::ProjectOverview, model::app_state::AppState};

#[derive(Props, Clone)]
pub struct ProjectListProps {
    pub app_state: Signal<AppState>,
    pub on_delete_clicked: EventHandler<Project>,
}

impl PartialEq for ProjectListProps {
    fn eq(&self, other: &Self) -> bool {
        self.app_state == other.app_state
    }
}

#[component]
pub fn ProjectList(props: ProjectListProps) -> Element {
    rsx! {
        style { { include_str!("./project_list.css") } }
        div { class: "ProjectList",
            for project in props.app_state.read().projects.iter() {
                ProjectOverview {
                    project: project.clone(),
                    on_delete_clicked: move |project: Project| {
                        props.on_delete_clicked.call(project);
                    }
                }
            }
        }
    }
}
