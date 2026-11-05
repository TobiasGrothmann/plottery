use dioxus::prelude::*;
use path_absolutize::Absolutize;
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
    let preview_image = props.project.get_editor_thumbnail_path();
    let preview_path = preview_image
        .absolutize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let preview_exists = preview_image.exists();
    let dir = props.project.dir.clone();
    let name = props.project.config.name.clone();
    let modified = format_datetime_to_relative(&props.project.config.last_modified_date);

    let open_project = props.project.clone();

    rsx! {
        style { { include_str!("./project_card.css") } }
        div { class: "ProjectCard",
            title: "{dir.to_string_lossy()}",
            onclick: move |_| props.on_open.call(open_project.clone()),
            div { class: "card_preview",
                if preview_exists {
                    Image { img_path: preview_path, class: "card_image" }
                } else {
                    div { class: "card_no_preview", p { "no preview yet" } }
                }
            }
            div { class: "card_body",
                div { class: "card_text",
                    h2 { class: "card_name", "{name}" }
                    p { class: "card_meta",
                        span { "{modified}" }
                        span { class: "card_meta_dim", " ago" }
                    }
                }
            }
        }
    }
}
