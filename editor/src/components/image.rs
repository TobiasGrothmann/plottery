use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

#[derive(PartialEq, Props)]
pub struct ImageProps {
    pub class: String,
    pub img_path: String,
}

pub fn Image(cx: Scope<ImageProps>) -> Element {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    cx.render(rsx!(
        style { include_str!("./image.css") }
        div { class: cx.props.class.as_str(),
            img {
                class: "image",
                src: "{cx.props.img_path.as_str()}?{ms}",
            }
        }
    ))
}
