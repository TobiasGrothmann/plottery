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
        params_editor::param_tree::{get_param_value_mut_by_path, leaf_value_mut},
        pick_target::PickTarget,
    },
    util::preview_zoom::PreviewZoom,
};

#[derive(PartialEq, Props, Clone)]
pub struct PreviewStageProps {
    /// The live preview, `None` until the first render.
    pub svg: SyncSignal<Option<String>>,
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

/// The drawing in the middle of the editor, with click-to-pick for V2 params.
#[component]
pub fn PreviewStage(props: PreviewStageProps) -> Element {
    let ctx = use_context::<EditorCtx>();
    let mut fullscreen = props.fullscreen;
    let mut zoom = props.zoom;
    // The cached preview stays below the live svg until that is on screen, or the swap flickers.
    let mut svg_on_screen = use_signal(|| false);

    let mut pick_target = use_context::<PickTarget>();
    let mut project_params = ctx.project_params;
    let onpick = move |point: V2| {
        let Some(path) = pick_target.take() else {
            return;
        };
        let mut new_params = project_params.read().clone();
        match get_param_value_mut_by_path(&mut new_params.list, &path) {
            Some(value) => match leaf_value_mut(value) {
                ProjectParamValue::V2(v) => *v = point,
                _ => {
                    tracing::error!("Pick target is not a V2 param: {:?}", path);
                    return;
                }
            },
            None => {
                tracing::error!("Pick target not found: {:?}", path);
                return;
            }
        }
        project_params.set(new_params);
        ctx.trigger_run();
    };

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
                    svg,
                    layer_bbox: props.layer_bbox,
                    pick_active: pick_target.is_active(),
                    onpick,
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
