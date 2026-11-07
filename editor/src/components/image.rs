use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_lib::{Rect, V2};
use std::time::UNIX_EPOCH;

use crate::util::{format_svg, preview_coords::preview_click_to_layer};

#[derive(PartialEq, Props, Clone)]
pub struct SVGImageProps {
    pub svg: String,
    /// Bounding box of the layer the svg was made from, needed to map clicks back.
    pub layer_bbox: Option<Rect>,
    /// Shows a crosshair and reports clicks through `onpick`.
    pub pick_active: bool,
    pub onpick: EventHandler<V2>,
}

#[component]
pub fn SVGImage(props: SVGImageProps) -> Element {
    let svg_data_url = format_svg(props.svg.as_bytes());
    let mut img = use_signal(|| None::<Rc<MountedData>>);
    let cursor = if props.pick_active {
        "crosshair"
    } else {
        "default"
    };

    rsx!(
        div { class: "Image",
            style: "height: 100%; width: 100%; flex: 1; display: flex; align-items: flex-start; justify-content: flex-start;",

            img {
                src: "{svg_data_url}",
                style: "flex: 1; max-width: 100%; max-height: 100%; cursor: {cursor};",
                onmounted: move |event| img.set(Some(event.data())),
                onclick: move |event| {
                    if !props.pick_active {
                        return;
                    }
                    let (Some(img), Some(bbox)) = (img(), props.layer_bbox) else { return };
                    let click = event.data.element_coordinates();
                    let click = V2::new(click.x as f32, click.y as f32);
                    let onpick = props.onpick;
                    spawn(async move {
                        match img.get_client_rect().await {
                            Ok(rect) => {
                                let element = V2::new(rect.size.width as f32, rect.size.height as f32);
                                if let Some(point) = preview_click_to_layer(click, element, bbox) {
                                    onpick.call(point);
                                }
                            }
                            Err(e) => tracing::error!("Failed to get preview size: {:?}", e),
                        }
                    });
                },
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
