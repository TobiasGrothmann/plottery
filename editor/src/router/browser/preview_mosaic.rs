use dioxus::prelude::*;

use crate::{components::image::Image, util::format_svg};

#[derive(PartialEq, Props, Clone)]
pub struct PreviewMosaicProps {
    /// Up to four absolute paths of preview images.
    pub previews: Vec<String>,
}

#[component]
pub fn PreviewMosaic(props: PreviewMosaicProps) -> Element {
    let previews: Vec<String> = props.previews.into_iter().take(4).collect();

    rsx! {
        style { { include_str!("./preview_mosaic.css") } }
        if previews.is_empty() {
            div { class: "PreviewMosaic mosaic_empty",
                img { src: "{format_svg(include_bytes!(\"../../../public/icons/folder.svg\"))}" }
            }
        } else {
            div { class: "PreviewMosaic",
                for preview in previews {
                    div { class: "mosaic_cell",
                        Image { img_path: preview }
                    }
                }
            }
        }
    }
}
