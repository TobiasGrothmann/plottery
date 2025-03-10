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

    let layer_name = match props.layer_tree_ref.props.name.clone() {
        Some(name) => name,
        None => "".to_owned(),
    };
    let num_shapes_text = if props.layer_tree_ref.num_shapes == 0 {
        "".to_owned()
    } else {
        format!("({})", props.layer_tree_ref.num_shapes)
    };

    let indentation_size = "21px";
    let margin_left_style = format!(
        "margin-left: calc({} * {});",
        props.indices.len(),
        indentation_size
    );

    let indices_clone = props.indices.clone();
    rsx! {
        div { class: "shapes_and_sublayers",
            div {
                style: margin_left_style.clone(),
                onclick: move |event| {
                    props.on_change_shapes_visible.call((
                        indices_clone.clone(),
                        !props.layer_tree_ref.shapes_visible,
                    ));
                    event.stop_propagation();
                },

                div { class: format!("row {}", layer_class),
                    div {
                        class: "layer_color_indicator",
                        style: "background-color: {layer_color.hex()};",
                        img { class: "eye",
                            src: "{format_svg(eye_icon_shapes.as_slice())}",
                            height: "17px",
                        }
                    }
                    p {"{layer_name}"}
                    p { style: "color: #BBB; margin-left: 6px;",
                        "{num_shapes_text}"
                    }
                }
            }
            if !props.layer_tree_ref.sublayers.is_empty() {
                div { class: "row",
                    style: margin_left_style.clone(),
                    onclick: move |event| {
                        props.on_change_sublayers_visible.call((
                            props.indices.clone(),
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
                    div { class: "sublayers",
                        style: margin_left_style.clone(),
                        {
                            props.layer_tree_ref.sublayers.into_iter().enumerate().map(|(i, sublayer)| {
                                let mut indices_sublayer = props.indices.clone();
                                indices_sublayer.insert(0, i);
                                rsx! {
                                    LayerEditorLayer {
                                        layer_tree_ref: sublayer,
                                        indices: indices_sublayer,
                                        on_change_shapes_visible: props.on_change_shapes_visible,
                                        on_change_sublayers_visible: props.on_change_sublayers_visible,
                                    }
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}
