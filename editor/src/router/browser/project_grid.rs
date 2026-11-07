use std::{collections::BTreeSet, path::PathBuf};

use dioxus::prelude::*;
use plottery_project::Project;

use crate::{
    components::{
        icon::{Icon, IconName},
        ui::{IconButton, Size},
    },
    model::folder_tree::FolderNode,
    router::browser::project_card::ProjectCard,
};

#[derive(PartialEq, Props, Clone)]
pub struct ProjectGridProps {
    pub node: FolderNode,
    pub thumbnail_generation: u32,
    pub collapsed: BTreeSet<PathBuf>,
    pub on_open: EventHandler<Project>,
    /// Called with a subfolder's path to show only that folder.
    pub on_select_folder: EventHandler<PathBuf>,
    pub on_toggle_section: EventHandler<PathBuf>,
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
                    p { class: "mono hint", "{props.node.path.to_string_lossy()}" }
                }
            }
            for section in sections.iter() {
                {
                    let is_subfolder = !section.label.is_empty();
                    let label = if is_subfolder { section.label.clone() } else { props.node.label.clone() };
                    let is_collapsed = props.collapsed.contains(&section.path);
                    let toggle_path = section.path.clone();
                    let select_path = section.path.clone();
                    rsx! {
                        section { class: "grid_section", key: "{section.path.to_string_lossy()}",
                            div { class: "grid_section_head",
                                button {
                                    r#type: "button",
                                    class: "sec-head",
                                    title: "{section.path.to_string_lossy()}",
                                    "aria-expanded": if is_collapsed { "false" } else { "true" },
                                    onclick: move |_| props.on_toggle_section.call(toggle_path.clone()),
                                    Icon {
                                        name: IconName::ChevronDown,
                                        class: if is_collapsed { "sec-chevron rot-90" } else { "sec-chevron" },
                                    }
                                    Icon { name: IconName::Folder, class: "sec-folder" }
                                    span { class: "sec-label", "{label}" }
                                    span { class: "count", "{section.projects.len()}" }
                                }
                                if is_subfolder {
                                    IconButton {
                                        icon: IconName::ChevronRight,
                                        label: "Show only {label}",
                                        size: Size::Sm,
                                        onclick: move |_| props.on_select_folder.call(select_path.clone()),
                                    }
                                }
                            }
                            if !is_collapsed {
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
}
