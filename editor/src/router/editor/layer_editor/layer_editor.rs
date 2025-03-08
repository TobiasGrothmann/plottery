use dioxus::prelude::*;

use crate::router::editor::{editor::LayerChangeWrapper, running_state::RunningState};

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorProps {
    layer: SyncSignal<LayerChangeWrapper>,
    running_state: SyncSignal<RunningState>,
}

#[component]
pub fn LayerEditor(props: LayerEditorProps) -> Element {
    let layer_option = props.layer.read().layer.clone();
    match layer_option {
        Some(layer) => {
            return rsx! {
                style { { include_str!("layer_editor.css") } }
                div { class: "LayerEditor",
                    LayerEditorLayer {
                        layer: LayerChangeWrapper{
                            layer: Some(layer),
                            change_counter: 0
                        },
                        recursion_depth: 0
                    }
                }
            };
        }
        None => rsx!("no layer"),
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorLayerProps {
    layer: LayerChangeWrapper,
    recursion_depth: usize,
}

fn LayerEditorLayer(props: LayerEditorLayerProps) -> Element {
    let layer = props.layer.layer.unwrap();
    let sub_layers = layer.sublayers;
    println!("recursion_depth: {}", props.recursion_depth);
    rsx! {
        div {
            style: "margin-left: calc({props.recursion_depth} * var(--gap-1));",

            "Layer"

            for sub_layer in sub_layers {
                LayerEditorLayer {
                    layer: LayerChangeWrapper{
                        layer: Some(sub_layer),
                        change_counter: 0
                    },
                    recursion_depth: props.recursion_depth + 1,
                }
            }
        }
    }
}
