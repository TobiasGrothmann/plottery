use dioxus::prelude::*;
use dioxus_logger::tracing;
use plottery_lib::{Rect, V2};
use plottery_runtime::project_param_value::ProjectParamValue;

use crate::{
    components::{
        icon::IconName,
        image::*,
        loading_spinner::Loading,
        ui::{Button, IconButton, Variant},
    },
    router::editor::{
        editor_ctx::EditorCtx,
        editor_data::PreviewSvg,
        params_editor::{param_tree::get_param_value_by_path, params_editor::edit_leaf_at},
        pick_target::{PickPart, PickTarget},
    },
    util::preview_zoom::PreviewZoom,
};

#[derive(PartialEq, Props, Clone)]
pub struct PreviewStageProps {
    /// The live preview, `None` until the first render.
    pub svg: SyncSignal<Option<PreviewSvg>>,
    /// Shown below the live preview until that is on screen.
    pub cached_preview: Option<String>,
    pub layer_bbox: Option<Rect>,
    pub redraw_counter: u32,
    pub busy: bool,
    /// A layer is loaded or still loading.
    pub has_drawing: bool,
    pub fullscreen: Signal<bool>,
    pub zoom: Signal<PreviewZoom>,
}

/// The drawing in the middle of the editor, with click-to-pick for V2 params and Rect corners.
#[component]
pub fn PreviewStage(props: PreviewStageProps) -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut fullscreen = props.fullscreen;
    let mut zoom = props.zoom;
    // The cached preview stays below the live svg until that is on screen, or the swap flickers.
    let mut svg_on_screen = use_signal(|| false);

    let mut pick_target = use_context::<PickTarget>();
    let project_params = ctx.project_params;
    let onpick = move |point: V2| {
        let Some(pick) = pick_target.take() else {
            return;
        };
        let set_point = |value: &mut ProjectParamValue| match (value, pick.part) {
            (ProjectParamValue::V2(v), PickPart::Point) => *v = point,
            (ProjectParamValue::Rect(rect), PickPart::BottomLeft) => {
                *rect = Rect::new(point, rect.tr())
            }
            (ProjectParamValue::Rect(rect), PickPart::TopRight) => {
                *rect = Rect::new(rect.bl(), point)
            }
            (value, part) => tracing::error!(
                "Can't pick {part:?} of a {} param: {:?}",
                value.type_name(),
                pick.path
            ),
        };
        edit_leaf_at(&pick.path, set_point, true);
    };

    let mark = pick_target.marked().and_then(|pick| {
        let value = get_param_value_by_path(&project_params.read().list, &pick.path)?;
        let value = match value {
            ProjectParamValue::Optional(optional) if !optional.enabled => return None,
            ProjectParamValue::Optional(optional) => *optional.value,
            other => other,
        };
        match (value, pick.part) {
            (ProjectParamValue::V2(v), _) => Some(PreviewMark::Point(v)),
            (ProjectParamValue::Rect(rect), part) => Some(PreviewMark::Rect {
                rect,
                corner: match part {
                    PickPart::BottomLeft => Some(rect.bl()),
                    PickPart::TopRight => Some(rect.tr()),
                    PickPart::Point => None,
                },
            }),
            _ => None,
        }
    });

    let busy = props.busy;
    rsx! {
        section {
            class: match (busy, fullscreen()) {
                (false, false) => "preview",
                (true, false) => "preview busy",
                (false, true) => "preview fullscreen",
                (true, true) => "preview busy fullscreen",
            },
            if fullscreen() {
                div { class: "fullscreen-close",
                    IconButton {
                        icon: IconName::X,
                        label: "Close fullscreen",
                        title: "Close fullscreen (Esc)",
                        onclick: move |_| {
                            fullscreen.set(false);
                            zoom.set(PreviewZoom::default());
                        },
                    }
                }
            }
            if let Some(img_path) = props.cached_preview.clone().filter(|_| !svg_on_screen()) {
                div { class: "preview-stage",
                    Image {
                        class: "paper paper-file",
                        img_path,
                        redraw_counter: props.redraw_counter,
                    }
                }
            }
            if let Some(svg) = (props.svg)() {
                SVGImage {
                    src: svg.src,
                    layer_bbox: props.layer_bbox,
                    pick_active: pick_target.is_active(),
                    onpick,
                    mark,
                    zoom,
                    onready: move |_| svg_on_screen.set(true),
                }
                if busy {
                    Loading {}
                }
            } else if props.cached_preview.is_some() {
                if busy {
                    Loading {}
                }
            } else if props.has_drawing {
                Loading {}
            } else {
                div { class: "preview-empty",
                    p { "No preview yet" }
                    Button {
                        variant: Variant::Primary,
                        icon: IconName::Play,
                        onclick: move |_| ctx.trigger_run(),
                        "Run"
                    }
                }
                if busy {
                    Loading {}
                }
            }
        }
    }
}
