use dioxus::prelude::*;
use plottery_lib::{Rect, V2};
use serde::Deserialize;
use std::time::UNIX_EPOCH;

use crate::{
    components::{
        icon::IconName,
        ui::{IconButton, Size},
    },
    util::{
        format_svg,
        preview_coords::preview_click_to_layer,
        preview_zoom::{PreviewZoom, ZoomViewport, CLICK_ZOOM},
    },
};

#[derive(PartialEq, Props, Clone)]
pub struct SVGImageProps {
    pub svg: String,
    /// Bounding box of the layer the svg was made from, needed to map clicks back.
    pub layer_bbox: Option<Rect>,
    /// Shows a crosshair and reports clicks through `onpick`.
    pub pick_active: bool,
    pub onpick: EventHandler<V2>,
    pub zoom: Signal<PreviewZoom>,
}

/// An input event from `preview_input.js`; positions are relative to the stage centre.
#[derive(Deserialize)]
struct PreviewInput {
    kind: String,
    x: f32,
    y: f32,
    paper_w: f32,
    paper_h: f32,
    stage_w: f32,
    stage_h: f32,
    factor: f32,
    dx: f32,
    dy: f32,
}

impl PreviewInput {
    fn at(&self) -> V2 {
        V2::new(self.x, self.y)
    }

    fn view(&self) -> ZoomViewport {
        ZoomViewport {
            paper: V2::new(self.paper_w, self.paper_h),
            stage: V2::new(self.stage_w, self.stage_h),
        }
    }
}

#[derive(Clone, Copy)]
struct PickState {
    layer_bbox: Option<Rect>,
    active: bool,
    onpick: EventHandler<V2>,
}

#[component]
pub fn SVGImage(props: SVGImageProps) -> Element {
    let svg_data_url = format_svg(props.svg.as_bytes());
    let mut zoom = props.zoom;
    let mut animate = use_signal(|| false);
    // The input loop outlives this render, so it reads the latest pick props from here.
    let mut pick = use_hook(|| {
        CopyValue::new(PickState {
            layer_bbox: props.layer_bbox,
            active: props.pick_active,
            onpick: props.onpick,
        })
    });
    *pick.write() = PickState {
        layer_bbox: props.layer_bbox,
        active: props.pick_active,
        onpick: props.onpick,
    };

    let mut set_animate = move |on: bool| {
        if *animate.peek() != on {
            animate.set(on);
        }
    };
    let start_input = move |_| {
        spawn(async move {
            let mut eval = document::eval(include_str!("preview_input.js"));
            while let Ok(input) = eval.recv::<PreviewInput>().await {
                let view = input.view();
                let current = *zoom.peek();
                let next = match input.kind.as_str() {
                    "zoom" => {
                        set_animate(false);
                        current.zoom_at(input.factor, input.at(), view)
                    }
                    "pan" => {
                        set_animate(false);
                        current.pan_by(V2::new(input.dx, input.dy), view)
                    }
                    "click" => {
                        let on_paper = current.stage_to_paper(input.at(), view.paper);
                        let state = *pick.peek();
                        if state.active {
                            let point = state.layer_bbox.and_then(|bbox| {
                                preview_click_to_layer(on_paper, view.paper, bbox)
                            });
                            if let Some(point) = point {
                                state.onpick.call(point);
                            }
                            continue;
                        }
                        let inside = on_paper.x >= 0.0
                            && on_paper.y >= 0.0
                            && on_paper.x <= view.paper.x
                            && on_paper.y <= view.paper.y;
                        set_animate(true);
                        match (current.is_fit(), inside) {
                            (true, true) => current.zoom_at(CLICK_ZOOM, input.at(), view),
                            (true, false) => continue,
                            (false, _) => PreviewZoom::default(),
                        }
                    }
                    _ => continue,
                };
                if next != current {
                    zoom.set(next);
                }
            }
        });
    };

    let size = props
        .layer_bbox
        .map(|bbox| bbox.size())
        .filter(|size| size.x > 0.0 && size.y > 0.0);
    let fit = match size {
        Some(size) => {
            let ratio = size.x / size.y;
            format!("width: min(100cqw, calc(100cqh * {ratio})); height: min(100cqh, calc(100cqw / {ratio}));")
        }
        None => "max-width: 100%; max-height: 100%;".to_string(),
    };
    let current = zoom();
    let transform = format!(
        "transform: translate({}px, {}px) scale({});",
        current.pan.x, current.pan.y, current.scale
    );
    let (stage_cursor, paper_cursor) = match (props.pick_active, current.is_fit()) {
        (true, _) => ("default", "crosshair"),
        (false, true) => ("default", "zoom-in"),
        (false, false) => ("zoom-out", "zoom-out"),
    };
    let paper_class = if animate() { "paper animate" } else { "paper" };
    let percent = (current.scale * 100.0).round();

    rsx!(
        div {
            id: "preview-stage",
            class: "preview-stage zoomable",
            style: "cursor: {stage_cursor};",
            onmounted: start_input,
            img {
                class: paper_class,
                src: "{svg_data_url}",
                draggable: "false",
                style: "{fit} {transform} cursor: {paper_cursor};",
            }
            if !current.is_fit() {
                div { class: "zoom-ctl",
                    span { class: "mono", "{percent}%" }
                    IconButton {
                        icon: IconName::Fit,
                        label: "Fit to view",
                        title: "Fit to view (Esc)",
                        size: Size::Sm,
                        onclick: move |_| {
                            set_animate(true);
                            zoom.set(PreviewZoom::default());
                        },
                    }
                }
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

    rsx!(img {
        class: "{props.class}",
        src: "{props.img_path.as_str()}?{cache_key}",
        loading: "lazy",
    })
}
