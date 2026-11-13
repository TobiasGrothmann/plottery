use dioxus::prelude::*;
use plottery_project::Project;

use crate::{components::image::Image, util::format_datetime_to_relative};

#[derive(Props, Clone)]
pub struct ProjectCardProps {
    pub project: Project,
    /// Bumped when thumbnails were (re)generated so the card re-checks the file.
    pub thumbnail_generation: u32,
    pub on_open: EventHandler<Project>,
}

impl PartialEq for ProjectCardProps {
    fn eq(&self, other: &Self) -> bool {
        self.project.get_project_config_path() == other.project.get_project_config_path()
            && self.project.config.last_modified_date == other.project.config.last_modified_date
            && self.thumbnail_generation == other.thumbnail_generation
    }
}

#[component]
pub fn ProjectCard(props: ProjectCardProps) -> Element {
    // The thumbnail is cached per folder, so without a layer it would be a former project's.
    let thumbnail = props
        .project
        .get_editor_thumbnail_path()
        .ok()
        .filter(|path| path.exists() && props.project.get_editor_layer_path().exists())
        .map(|path| path.to_string_lossy().to_string());
    let dir = props.project.dir.clone();
    let name = props.project.config.name.clone();
    let modified = format_datetime_to_relative(&props.project.config.last_modified_date);

    let open_project = props.project.clone();

    let key_project = props.project.clone();

    rsx! {
        style { { include_str!("./project_card.css") } }
        div {
            class: "card",
            role: "button",
            tabindex: "0",
            title: "{dir.to_string_lossy()}",
            onclick: move |_| props.on_open.call(open_project.clone()),
            onkeydown: move |event| {
                if event.key() == Key::Enter {
                    props.on_open.call(key_project.clone());
                }
            },
            div { class: "thumb",
                if let Some(img_path) = thumbnail {
                    Image { img_path, class: "thumb_paper" }
                } else {
                    span { class: "thumb_empty", "No preview yet" }
                }
            }
            div { class: "card_body",
                span { class: "card_name", "{name}" }
                span { class: "card_meta", "{modified} ago" }
            }
        }
    }
}
