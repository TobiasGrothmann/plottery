use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::{util::format_datetime_to_relative, components::image::Image, routes::Route};

#[derive(PartialEq, Props)]
pub struct ProjectOverviewProps {
    pub project: Project,
}

#[component]
pub fn ProjectOverview(cx: Scope<ProjectOverviewProps>) -> Element {
    let project_exists = cx.props.project.exists();
    let preview_image = cx.props.project.get_preview_image_path();

    let folder_logo_path = if cfg!(target_os = "windows") {
        "icons/explorer.svg"
    } else if cfg!(target_os = "macos") {
        "icons/finder.svg"
    } else {
        "icons/linux_folder.svg"
    };
    
    cx.render(rsx! {
        style { include_str!("./project_overview.css") }
        div { class: "ProjectOverview card",
            if project_exists {
                cx.render(rsx!(
                    div { class: "preview",
                        if preview_image.exists() {
                            cx.render(rsx!(
                                Image {
                                    img_path: preview_image.absolutize().unwrap().to_string_lossy().to_string(),
                                    redraw_counter: 0
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
                if project_exists {
                    cx.render(rsx!(
                        div { class: "open_actions",
                            button { class: "icon_button",
                                onclick: move |_event| {
                                    let project_dir = cx.props.project.dir.clone();
                                    std::process::Command::new("code")
                                        .arg(project_dir)
                                        .spawn()
                                        .unwrap();
                                },
                                img { src: "icons/vscode.svg" }
                            }
                            button { class: "icon_button",
                                onclick: move |_event| {
                                    opener::reveal(cx.props.project.dir.clone()).unwrap();
                                },
                                img { src: "{folder_logo_path}" }
                            }
                        }
                    ))
                }
            }
            div { class: "actions",
                button { class: "delete_button",
                    onclick: move |_event| {
                    },
                    img { src: "icons/delete.svg" }
                }
                if project_exists {
                    cx.render(rsx!(
                        button { class: "icon_button",
                            onclick: move |_event| {
                                let nav = use_navigator(cx);
                                nav.push(Route::Editor {
                                    project_path: cx.props.project.get_project_config_path().absolutize().unwrap().to_string_lossy().to_string()
                                });
                            },
                            img { src: "icons/forward.svg" }
                        }
                    ))
                }
            }
        }
    })
}
