use std::path::PathBuf;

use dioxus::prelude::*;
use plottery_project::Project;

use crate::{
    model::folder_tree::FolderNode, router::browser::project_card::ProjectCard, util::format_svg,
};

#[derive(PartialEq, Props, Clone)]
pub struct ProjectGridProps {
    pub node: FolderNode,
    pub thumbnail_generation: u32,
    pub on_open: EventHandler<Project>,
    /// Called with a subfolder's path when its section header is clicked.
    pub on_select_folder: EventHandler<PathBuf>,
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
            for section in sections.iter() {
                {
                    let is_subfolder = !section.label.is_empty();
                    let label = if is_subfolder { section.label.clone() } else { props.node.label.clone() };
                    let header_class = if is_subfolder {
                        "grid_section_header grid_section_header_link"
                    } else {
                        "grid_section_header"
                    };
                    let section_path = section.path.clone();
                    rsx! {
                        div { class: "grid_section",
                            div { class: "{header_class}",
                                title: "{section.path.to_string_lossy()}",
                                onclick: move |_| {
                                    if is_subfolder {
                                        props.on_select_folder.call(section_path.clone());
                                    }
                                },
                                img { class: "grid_section_icon",
                                    src: "{format_svg(include_bytes!(\"../../../public/icons/folder.svg\"))}"
                                }
                                h3 { class: "grid_section_title", "{label}" }
                                p { class: "grid_section_count", "{section.projects.len()}" }
                            }
                            div { class: "grid_cards",
                                for project in section.projects.iter() {
                                    ProjectCard {
                                        project: (*project).clone(),
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
    }
}
