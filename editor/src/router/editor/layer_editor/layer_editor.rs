use dioxus::prelude::*;

use crate::router::editor::running_state::RunningState;
use crate::util::format_svg;

use super::layer_tree_ref::LayerTreeReference;

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorProps {
    layer_tree_ref: SyncSignal<Option<LayerTreeReference>>,
    running_state: SyncSignal<RunningState>,
}

#[component]
pub fn LayerEditor(props: LayerEditorProps) -> Element {
    let mut layer_tree_ref_copy = props.layer_tree_ref;
    match &*props.layer_tree_ref.read() {
        Some(layer_tree_ref) => {
            return rsx! {
                style { { include_str!("layer_editor.css") } }
                div { class: "LayerEditor",
                    LayerEditorLayer {
                        layer_tree_ref: layer_tree_ref.clone(),
                        recursion_depth: 0,
                        layer_index: 0,
                        on_change: move |(depth, index, visible, subs_visible): (usize, usize, bool, bool)| {
                            layer_tree_ref_copy.write().as_mut().unwrap().toggle_visible(depth, index);
                        }
                    }
                }
            };
        }
        None => rsx!(),
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorLayerProps {
    layer_tree_ref: LayerTreeReference,
    recursion_depth: usize,
    layer_index: usize,
    on_change: Callback<(usize, usize, bool, bool)>,
}

fn LayerEditorLayer(props: LayerEditorLayerProps) -> Element {
    let layer_color = props.layer_tree_ref.props.color.unwrap();
    let layer_class = if props.layer_tree_ref.is_visible {
        ""
    } else {
        "layer_hidden"
    };
    let eye_open = include_bytes!("../../../../public/icons/eye_open.svg");
    let eye_closed = include_bytes!("../../../../public/icons/eye_closed.svg");
    let eye_icon = if props.layer_tree_ref.is_visible {
        eye_open.to_vec()
    } else {
        eye_closed.to_vec()
    };

    rsx! {
        div {
            style: "margin-left: calc({props.recursion_depth} * var(--gap-1));",
            onclick: move |event| {
                props.on_change.call((
                    props.recursion_depth,
                    props.layer_index,
                    props.layer_tree_ref.is_visible,
                    props.layer_tree_ref.are_sublayers_visible
                ));
                event.stop_propagation();
            },

            div { class: format!("LayerEditorLayer {}", layer_class),
                img { src: "{format_svg(eye_icon.as_slice())}", height: "16px" }
                div {
                    style: "display: inline-block; width: 12px; height: 12px; border-radius: 50%; background-color: {layer_color.hex()}; border: 1px solid white;"
                }
                "Layer {props.layer_index}"
            }

            for (i, sub_layer) in props.layer_tree_ref.sub_layers.into_iter().enumerate() {
                LayerEditorLayer {
                    layer_tree_ref: sub_layer,
                    recursion_depth: props.recursion_depth + 1,
                    layer_index: i,
                    on_change: props.on_change
                }
            }
        }
    }
}
