use dioxus::prelude::*;
use plottery_project::Project;

use crate::{model::folder_tree::FolderNode, router::browser::project_card::ProjectCard};

#[derive(PartialEq, Props, Clone)]
pub struct ProjectGridProps {
    pub node: FolderNode,
    pub thumbnail_generation: u32,
    pub on_open: EventHandler<Project>,
}

#[component]
pub fn ProjectGrid(props: ProjectGridProps) -> Element {
    let sections = props.node.sections();

    rsx! {
        style { { include_str!("./project_grid.css") } }
        div { class: "ProjectGrid",
            if sections.is_empty() {
                div { class: "grid_empty",
                    p { "No projects found in" }
                    p { class: "grid_empty_path", "{props.node.path.to_string_lossy()}" }
                }
            }
            for section in sections {
                div { class: "grid_section",
                    if !section.label.is_empty() {
                        h3 { class: "grid_section_header",
                            title: "{section.path.to_string_lossy()}",
                            "{section.label}"
                        }
                    }
                    div { class: "grid_cards",
                        for project in section.projects {
                            ProjectCard {
                                project: project.clone(),
                                thumbnail_generation: props.thumbnail_generation,
                                on_open: props.on_open,
                            }
                        }
                    }
                }
            }
        }
    }
}
