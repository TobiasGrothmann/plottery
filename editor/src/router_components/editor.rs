use std::path::PathBuf;

use dioxus::prelude::*;
use dioxus_router::hooks::use_navigator;
use path_absolutize::Absolutize;
use plottery_lib::Layer;
use plottery_project::Project;

use crate::components::image::Image;

fn get_svg_path(project: &Project) -> PathBuf {
    project.get_preview_image_path()
}

#[component]
pub fn Editor(cx: Scope, project_path: String) -> Element {
    let project = use_state(cx, || {
        let path = PathBuf::from(project_path.clone());
        Project::load_from_file(path).unwrap()
    });

    let layer = use_state(cx, || None as Option<Layer>);
    let svg_path = get_svg_path(project);
    let run_counter = use_state(cx, || 0);

    cx.render(rsx! {
        style { include_str!("./editor.css") }

        div { class: "Editor",
            div { class: "plot_header",
                button {
                    onclick: move |_event| {
                        let nav = use_navigator(cx);
                        nav.go_back();
                    },
                    "<-"
                }
                h1 {
                    "{project.config.name}"
                }
                div { class: "action_buttons",
                    button {
                        onclick: move |_event| {
                            project.compile(true).unwrap();
                        },
                        "compile"
                    }
                    button {
                        onclick: move |_event| {
                            let new_layer = project.run_code(true);
                            match new_layer {
                                Ok(new_layer) => {
                                    new_layer.write_svg(get_svg_path(project), 1.0).unwrap();
                                    layer.set(Some(new_layer));
                                    run_counter.set(*run_counter.get() + 1);
                                },
                                Err(e) => {log::error!("Error running code: {}", e)}
                            }
                        },
                        "run"
                    }
                }
            }

            div { class: "plot_and_params",
                div { class: "params",
                    p { "Parameters" } 
                }
                div { class: "plot",
                    if svg_path.exists() {
                        cx.render(rsx!(
                            Image {
                                img_path: get_svg_path(project).absolutize().unwrap().to_string_lossy().to_string(),
                                redraw_counter: *run_counter.get()
                            }
                        ))
                    } else {
                        cx.render(rsx!(
                            div { class: "err_box",
                                p { "SVG could not be found!" }
                            }
                        ))
                    }
                }
            }
        }
    })
}
