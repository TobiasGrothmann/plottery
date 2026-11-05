use dioxus::prelude::*;
use std::time::UNIX_EPOCH;

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
    #[props(default)]
    pub redraw_counter: u32,
    #[props(default)]
    pub class: String,
}

/// Modification time of the file in milliseconds, used as a cache-busting key.
fn file_mtime_millis(path: &str) -> u128 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[component]
pub fn Image(props: ImageProps) -> Element {
    let cache_key = format!(
        "{}-{}",
        file_mtime_millis(&props.img_path),
        props.redraw_counter
    );

    rsx!(
        div { class: "Image {props.class}",
            style: "height: 100%",

            img {
                src: "{props.img_path.as_str()}?{cache_key}",
                style: "height: 100%",
                loading: "lazy",
            }
        }
    )
}
