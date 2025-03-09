use plottery_lib::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerTreeReference {
    pub num_shapes: usize,
    pub props: LayerProps,
    pub sublayers: Vec<LayerTreeReference>,

    pub shapes_visible: bool,
    pub sublayers_visible: bool,
}

impl LayerTreeReference {
    pub fn new(layer: &Layer, parent_props: &LayerProps) -> Self {
        let props = parent_props.join_with_child(&layer.props);

        let sub_layers = layer
            .sublayers
            .iter()
            .map(|sub_layer| LayerTreeReference::new(sub_layer, &props))
            .collect();

        Self {
            num_shapes: layer.len(),
            props,
            sublayers: sub_layers,
            shapes_visible: true,
            sublayers_visible: true,
        }
    }

    pub fn set_shapes_visible(&mut self, depth: usize, index: usize, visible: bool) {
        if depth == 0 {
            self.shapes_visible = visible;
        } else {
            self.sublayers[index].set_shapes_visible(depth - 1, index, visible);
        }
    }

    pub fn set_sublayers_visible(&mut self, depth: usize, index: usize, visible: bool) {
        if depth == 0 {
            self.sublayers_visible = visible;
        } else {
            self.sublayers[index].set_sublayers_visible(depth - 1, index, visible);
        }
    }

    pub fn filter_layer_by_visibility(&self, layer: &Layer) -> Layer {
        let mut new_layer = Layer::new().with_props(layer.props);
        if self.shapes_visible {
            for shape in layer.iter() {
                new_layer.push(shape.clone());
            }
        }
        if self.sublayers_visible {
            for (sub_layer, layer_ref_tree) in layer.iter_sublayers().zip(self.sublayers.iter()) {
                if layer_ref_tree.sublayers_visible {
                    new_layer.push_layer(layer_ref_tree.filter_layer_by_visibility(sub_layer));
                }
            }
        }
        new_layer
    }
}
