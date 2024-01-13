use dioxus::prelude::*;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::{util::format_datetime_to_relative, components::image::Image};

#[derive(PartialEq, Props)]
pub struct ProjectOverviewProps {
    pub project: Project,
}

pub fn ProjectOverview(cx: Scope<ProjectOverviewProps>) -> Element {
    let project_exists = cx.props.project.exists();
    let preview_image = cx.props.project.get_preview_image_path();

    cx.render(rsx! {
        style { include_str!("./project_overview.css") }
        div { class: "project_overview card",
            div { class: "summary",
                if !project_exists { cx.render(rsx!(
                    div { class: "err_box",
                        p { "Project could not be found!" }
                    }
                ))}
                div {
                    h2 { cx.props.project.config.name.clone() }
                    p { class: "grey_text", cx.props.project.dir.absolutize().unwrap().to_string_lossy() }
                    p { 
                        span { format_datetime_to_relative(&cx.props.project.config.last_modified_date) }
                        span { class: "grey_text", " ago" }
                    }
                }
            }
            if project_exists {
                cx.render(rsx!(
                    div { class: "actions",
                        button {
                            onclick: move |_event| cx.props.project.compile(true).unwrap(),
                            "Compile"
                        }
                        button {
                            onclick: move |_event| {
                                let layer = cx.props.project.run_code(true).unwrap();
                                log::info!("layer: {:?}", layer);
                            },
                            "Run"
                        }
                    }
                    div { class: "preview",
                        if preview_image.is_some() {
                            cx.render(rsx!(
                                Image { class: "preview_image",
                                    img_path: preview_image.unwrap().absolutize().unwrap().to_string_lossy().to_string(),
                                }
                            ))
                        } else {
                            cx.render(rsx!(
                                div { class: "err_box",
                                    p { "Preview image could not be found!" }
                                }
                            ))
                        }
                    }
                ))
            }
        }
    })
}
