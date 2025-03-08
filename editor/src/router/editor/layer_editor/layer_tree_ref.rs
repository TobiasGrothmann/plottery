use plottery_lib::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerTreeReference {
    pub props: LayerProps,
    pub is_visible: bool,
    pub are_sublayers_visible: bool,
    pub sub_layers: Vec<LayerTreeReference>,
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
            props,
            sub_layers,
            is_visible: true,
            are_sublayers_visible: true,
        }
    }

    pub fn toggle_visible(&mut self, depth: usize, index: usize) {
        if depth == 0 {
            self.is_visible = !self.is_visible;
        } else {
            self.sub_layers[index].toggle_visible(depth - 1, index);
        }
    }

    pub fn filter_layer_by_visibility(&self, layer: &Layer) -> Layer {
        let mut new_layer = Layer::new().with_props(layer.props);
        if self.is_visible {
            for shape in layer.iter() {
                new_layer.push(shape.clone());
            }
        }
        for (sub_layer, layer_ref_tree) in layer.iter_sublayers().zip(self.sub_layers.iter()) {
            if layer_ref_tree.are_sublayers_visible {
                new_layer.push_layer(layer_ref_tree.filter_layer_by_visibility(sub_layer));
            }
        }
        new_layer
    }
}
