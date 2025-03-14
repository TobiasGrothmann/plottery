use dioxus::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::util::format_svg;

#[derive(PartialEq, Props, Clone)]
pub struct SVGImageProps {
    pub svg: String,
}

#[component]
pub fn SVGImage(props: SVGImageProps) -> Element {
    let svg_data_url = format_svg(props.svg.as_bytes());

    rsx!(
        div { class: "Image",
            style: "height: 100%; width: 100%; flex: 1; display: flex; align-items: flex-start; justify-content: flex-start;",

            img {
                src: "{svg_data_url}",
                style: "flex: 1; max-width: 100%; max-height: 100%;",
            }
        }
    )
}

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
        div { class: "Image",
            style: "height: 100%",

            img {
                src: "{props.img_path.as_str()}?{ms}",
                style: "height: 100%",
            }
        }
    )
}
