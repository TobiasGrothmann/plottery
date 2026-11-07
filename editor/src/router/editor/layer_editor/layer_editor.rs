use dioxus::prelude::*;

use crate::components::{
    icon::{Icon, IconName},
    ui::{IconButton, Size},
};
use crate::router::editor::running_state::RunningState;

use super::layer_tree_ref::LayerTreeReference;

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorProps {
    layer_tree_ref: SyncSignal<Option<LayerTreeReference>>,
    running_state: SyncSignal<RunningState>,
}

#[component]
pub fn LayerEditor(props: LayerEditorProps) -> Element {
    let mut layer_tree_ref_copy = props.layer_tree_ref;
    if layer_tree_ref_copy.read().is_none() {
        return rsx! {
            p { class: "hint layers-empty", "Run the project to see its layers." }
        };
    }

    let layer_tree_ref = layer_tree_ref_copy.read().as_ref().cloned().unwrap();
    rsx! {
        style { { include_str!("layer_editor.css") } }
        div { class: "LayerEditor",
            LayerEditorLayer {
                layer_tree_ref,
                indices: vec![],
                on_change_shapes_visible: move |(indices, visible): (Vec<usize>, bool)| {
                    let mut layer_guard = layer_tree_ref_copy.write();
                    let layer = layer_guard.as_mut().unwrap().get_by_indices(indices);
                    layer.shapes_visible = visible;
                },
                on_change_sublayers_visible: move |(indices, visible): (Vec<usize>, bool)| {
                    let mut layer_guard = layer_tree_ref_copy.write();
                    let layer = layer_guard.as_mut().unwrap().get_by_indices(indices);
                    layer.sublayers_visible = visible;
                }
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorLayerProps {
    layer_tree_ref: LayerTreeReference,
    indices: Vec<usize>,
    on_change_shapes_visible: Callback<(Vec<usize>, bool)>,
    on_change_sublayers_visible: Callback<(Vec<usize>, bool)>,
}

fn LayerEditorLayer(props: LayerEditorLayerProps) -> Element {
    let layer_color = props.layer_tree_ref.props_inheritable.color.unwrap();
    let shapes_visible = props.layer_tree_ref.shapes_visible;
    let sublayers_visible = props.layer_tree_ref.sublayers_visible;
    let layer_name = props.layer_tree_ref.props.name.clone();
    let shown_name = layer_name.clone().unwrap_or_else(|| "layer".to_string());
    let num_sublayers = props.layer_tree_ref.sublayers.len();
    let num_shapes = props.layer_tree_ref.num_shapes;
    let has_sublayers = num_sublayers > 0;
    let depth = props.indices.len();
    let row_class = if shapes_visible || num_shapes == 0 {
        "layer"
    } else {
        "layer off"
    };
    let eye_label = if shapes_visible {
        format!("Hide layer {shown_name}")
    } else {
        format!("Show layer {shown_name}")
    };

    let indices_for_shapes = props.indices.clone();
    let indices_for_sublayers = props.indices.clone();
    rsx! {
        div {
            class: "{row_class}",
            style: "padding-left: {6 + depth * 16}px;",
            onclick: move |event| {
                event.stop_propagation();
                if num_shapes == 0 {
                    return;
                }
                props.on_change_shapes_visible.call((
                    indices_for_shapes.clone(),
                    !shapes_visible,
                ));
            },
            if has_sublayers {
                button {
                    r#type: "button",
                    class: "layer-chevron",
                    "aria-label": if sublayers_visible { "Collapse and hide sublayers" } else { "Expand and show sublayers" },
                    "aria-expanded": if sublayers_visible { "true" } else { "false" },
                    onclick: move |event| {
                        event.stop_propagation();
                        props.on_change_sublayers_visible.call((
                            indices_for_sublayers.clone(),
                            !sublayers_visible,
                        ));
                    },
                    Icon {
                        name: IconName::ChevronDown,
                        class: if sublayers_visible { "" } else { "rot-90" },
                    }
                }
            } else {
                span { class: "layer-chevron" }
            }
            if num_shapes != 0 {
                span { class: "dot layer-dot", style: "background-color: {layer_color.hex()};" }
            }
            span { class: if layer_name.is_some() { "layer-name" } else { "layer-name hint" }, "{shown_name}" }
            if num_shapes != 0 {
                span { class: "count", "{num_shapes}" }
            }
            if has_sublayers && !sublayers_visible {
                span { class: "count", "{num_sublayers} hidden" }
            }
            if num_shapes != 0 {
                IconButton {
                    icon: if shapes_visible { IconName::Eye } else { IconName::EyeOff },
                    label: eye_label,
                    size: Size::Sm,
                }
            }
        }
        if has_sublayers && sublayers_visible {
            for (i, sublayer) in props.layer_tree_ref.sublayers.into_iter().enumerate() {
                {
                    let mut indices_sublayer = props.indices.clone();
                    indices_sublayer.insert(0, i);
                    rsx! {
                        LayerEditorLayer {
                            key: "{i}",
                            layer_tree_ref: sublayer,
                            indices: indices_sublayer,
                            on_change_shapes_visible: props.on_change_shapes_visible,
                            on_change_sublayers_visible: props.on_change_sublayers_visible,
                        }
                    }
                }
            }
        }
    }
}
