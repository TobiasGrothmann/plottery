use crate::util::format_svg;
use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::{components::image::Image, routes::Route, util::format_datetime_to_relative};

#[derive(Props)]
pub struct ProjectOverviewProps<'a> {
    pub project: Project,
    pub on_delete_clicked: EventHandler<'a, Project>,
}

impl PartialEq for ProjectOverviewProps<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.project == other.project
    }
}

#[component]
pub fn ProjectOverview<'a>(cx: Scope<'a, ProjectOverviewProps>) -> Element<'a> {
    let project_exists = cx.props.project.exists();
    let preview_image = cx.props.project.get_preview_image_path();

    let icon_folder = if cfg!(target_os = "windows") {
        format_svg(include_bytes!("../../public/icons/explorer.svg"))
    } else if cfg!(target_os = "macos") {
        format_svg(include_bytes!("../../public/icons/finder.svg"))
    } else {
        format_svg(include_bytes!("../../public/icons/linux_folder.svg"))
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
                                img { src: "{format_svg(include_bytes!(\"../../public/icons/vscode.svg\"))}" }
                            }
                            button { class: "icon_button",
                                onclick: move |_event| {
                                    opener::reveal(cx.props.project.dir.clone()).unwrap();
                                },
                                img { src: "{icon_folder}" }
                            }
                        }
                    ))
                }
            }
            div { class: "actions",
                button { class: "delete_button",
                    onclick: move |_event| { cx.props.on_delete_clicked.call(cx.props.project.clone()) },
                    img { src: "{format_svg(include_bytes!(\"../../public/icons/delete.svg\"))}" }
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
                            img { src: "{format_svg(include_bytes!(\"../../public/icons/forward.svg\"))}" }
                        }
                    ))
                }
            }
        }
    })
}
