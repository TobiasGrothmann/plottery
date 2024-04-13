use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct ImageProps {
    pub img_path: String,
    pub redraw_counter: u32,
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    rsx!(
        style { { include_str!("./image.css") } }
        div { class: "Image",
            img {
                src: "{props.img_path.as_str()}?{ms}",
            }
        }
    )
}
