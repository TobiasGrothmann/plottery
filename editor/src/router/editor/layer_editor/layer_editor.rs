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
    if layer_tree_ref_copy.read().is_none() {
        return rsx!();
    }

    let layer_tree_ref = layer_tree_ref_copy.read().as_ref().cloned().unwrap();
    rsx! {
        style { { include_str!("layer_editor.css") } }
        div { class: "LayerEditor",
            LayerEditorLayer {
                layer_tree_ref,
                recursion_depth: 0,
                layer_index: 0,
                on_change_shapes_visible: move |(depth, index, visible): (usize, usize, bool)|
                    layer_tree_ref_copy.write().as_mut().unwrap().set_shapes_visible(depth, index, visible),
                on_change_sublayers_visible: move |(depth, index, visible): (usize, usize, bool)|
                    layer_tree_ref_copy.write().as_mut().unwrap().set_sublayers_visible(depth, index, visible),
            }
        }
    }
}

#[derive(PartialEq, Props, Clone)]
pub struct LayerEditorLayerProps {
    layer_tree_ref: LayerTreeReference,
    recursion_depth: usize,
    layer_index: usize,
    on_change_shapes_visible: Callback<(usize, usize, bool)>,
    on_change_sublayers_visible: Callback<(usize, usize, bool)>,
}

fn LayerEditorLayer(props: LayerEditorLayerProps) -> Element {
    let layer_color = props.layer_tree_ref.props.color.unwrap();
    let layer_class = if props.layer_tree_ref.shapes_visible {
        ""
    } else {
        "hidden"
    };
    let eye_open = include_bytes!("../../../../public/icons/eye_open.svg");
    let eye_closed = include_bytes!("../../../../public/icons/eye_closed.svg");
    let eye_icon_shapes = if props.layer_tree_ref.shapes_visible {
        eye_open.to_vec()
    } else {
        eye_closed.to_vec()
    };

    let num_shapes_text = if props.layer_tree_ref.num_shapes == 0 {
        "".to_owned()
    } else {
        format!(
            "{} shape{}",
            props.layer_tree_ref.num_shapes,
            if props.layer_tree_ref.num_shapes != 1 {
                "s"
            } else {
                ""
            }
        )
    };

    let indentation_size = "21px";
    let margin_left_style = format!(
        "margin-left: calc({} * {});",
        props.recursion_depth, indentation_size
    );

    rsx! {
        div { class: "shapes_and_sublayers",
            div {
                style: margin_left_style.clone(),
                onclick: move |event| {
                    props.on_change_shapes_visible.call((
                        props.recursion_depth,
                        props.layer_index,
                        !props.layer_tree_ref.shapes_visible,
                    ));
                    event.stop_propagation();
                },

                div { class: format!("row {}", layer_class),
                    img { src: "{format_svg(eye_icon_shapes.as_slice())}", height: "16px" }
                    div {
                        style: "display: inline-block; width: 12px; height: 12px; border-radius: 50%; background-color: {layer_color.hex()}; border: 1px solid white;"
                    }
                    "{num_shapes_text}"
                }
            }
            if props.layer_tree_ref.sublayers.len() > 0 {
                div { class: "row",
                    style: margin_left_style.clone(),
                    onclick: move |event| {
                        props.on_change_sublayers_visible.call((
                            props.recursion_depth,
                            props.layer_index,
                            !props.layer_tree_ref.sublayers_visible,
                        ));
                        event.stop_propagation();
                    },
                    p { class: if props.layer_tree_ref.sublayers_visible { "" } else { "hidden" },
                        style: format!("height: 17px; margin-left: {}; margin-top: 2px;", indentation_size),
                        {if props.layer_tree_ref.sublayers_visible { "➴".to_owned() } else { format!("➵ {} hidden", props.layer_tree_ref.sublayers.len()) }}
                    }
                }

                if props.layer_tree_ref.sublayers_visible {
                    div {
                        style: margin_left_style.clone(),
                        for (i, sub_layer) in props.layer_tree_ref.sublayers.into_iter().enumerate() {
                            LayerEditorLayer {
                                layer_tree_ref: sub_layer,
                                recursion_depth: props.recursion_depth + 1,
                                layer_index: i,
                                on_change_shapes_visible: props.on_change_shapes_visible,
                                on_change_sublayers_visible: props.on_change_sublayers_visible,
                            }
                        }
                    }
                }
            }
        }
    }
}
