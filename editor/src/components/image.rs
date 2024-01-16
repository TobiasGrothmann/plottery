use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ImageProps {
    pub img_path: String,
    pub redraw_counter: u32,
}

#[component]
pub fn Image(cx: Scope<ImageProps>) -> Element {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    cx.render(rsx!(
        style { include_str!("./image.css") }
        div { class: "Image",
            img {
                src: "{cx.props.img_path.as_str()}?{ms}",
            }
        }
    ))
}
