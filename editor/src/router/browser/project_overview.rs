use crate::util::format_svg;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::{components::image::Image, routes::Route, util::format_datetime_to_relative};

#[derive(Props, Clone)]
pub struct ProjectOverviewProps {
    pub project: ReadSignal<Project>,
    pub on_delete_clicked: EventHandler<Project>,
}

impl PartialEq for ProjectOverviewProps {
    fn eq(&self, other: &Self) -> bool {
        self.project == other.project
    }
}

#[component]
pub fn ProjectOverview(props: ProjectOverviewProps) -> Element {
    let project_exists = props.project.read().exists();
    let preview_image = props.project.read().get_editor_preview_image_path();

    rsx! {
        style { { include_str!("./project_overview.css") } }
        div { class: "ProjectOverview",
            if project_exists {
                div { class: "preview",
                    if preview_image.exists() {
                        Image {
                            img_path: preview_image.absolutize().unwrap().to_string_lossy().to_string(),
                            redraw_counter: 0
                        }
                    } else {
                        div { class: "err_box",
                            p { "Preview image could not be found!" }
                        }
                    }
                }
            }
            div { class: "summary",
                if !project_exists {
                    div { class: "err_box",
                        p { "Project could not be found!" }
                    }
                }
                div {
                    h2 { "{props.project.read().config.name.clone()}" }
                    p { class: "grey_text", "{props.project.read().dir.absolutize().unwrap().to_string_lossy()}" }
                    p {
                        span { "{format_datetime_to_relative(&props.project.read().config.last_modified_date)}" }
                        span { class: "grey_text", " ago" }
                    }
                }
            }
            div { class: "actions",
                button { class: "delete_button",
                    onclick: move |_event| { props.on_delete_clicked.call(props.project.read().clone()) },
                    img { src: "{format_svg(include_bytes!(\"../../../public/icons/trash_white.svg\"))}" }
                }
                if project_exists {
                    button { class: "icon_button",
                        onclick: move |_event| {
                            let nav = use_navigator();
                            let path_str = props.project.read().get_project_config_path().absolutize().unwrap().to_string_lossy().to_string();
                            let project_path: Vec<String> = path_str.split('/').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
                            nav.push(Route::Editor {
                                project_path
                            });
                        },
                        img { src: "{format_svg(include_bytes!(\"../../../public/icons/forward.svg\"))}" }
                    }
                }
            }
        }
    }
}
