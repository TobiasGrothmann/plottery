use dioxus::prelude::*;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::util::format_datetime_to_relative;

#[derive(PartialEq, Props)]
pub struct ProjectOverviewProps {
    pub project: Project,
}

pub fn ProjectOverview(cx: Scope<ProjectOverviewProps>) -> Element {
    let project_exists = cx.props.project.exists();

    cx.render(rsx! {
        style { include_str!("./project_overview.css") }
        div { class: "project_overview card",
            div { class: "summary",
                if !project_exists { cx.render(rsx!(
                    div { class: "err_header",
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
        }
    })
}
