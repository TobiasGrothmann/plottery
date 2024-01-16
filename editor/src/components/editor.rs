use dioxus::prelude::*;
use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::components::image::Image;

#[derive(PartialEq, Props)]
pub struct EditorProps {
    pub project: Project,
}

#[component]
pub fn Editor(cx: Scope<EditorProps>) -> Element {
    let run_counter = use_state(cx, || 0_u32);
    let preview_image = cx.props.project.get_preview_image_path();

    cx.render(rsx!(
        style { include_str!("./editor.css") }
        div { class: "Editor",
            div { class: "actions",
                button {
                    onclick: move |_event| cx.props.project.compile(true).unwrap(),
                    "Compile"
                }
                button {
                    onclick: move |_event| {
                        cx.props.project.compile(true).unwrap();
                        let layer = cx.props.project.run_code(true).unwrap();
                        layer.write_svg(cx.props.project.get_preview_image_path(), 1.0).unwrap();
                        run_counter.set(run_counter.get() + 1);
                    },
                    "Run"
                }
            }
            div { class: "preview",
                if preview_image.exists() {
                    cx.render(rsx!(
                        Image {
                            img_path: preview_image.absolutize().unwrap().to_string_lossy().to_string(),
                            redraw_counter: *run_counter.get()
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
        }
    ))
}
