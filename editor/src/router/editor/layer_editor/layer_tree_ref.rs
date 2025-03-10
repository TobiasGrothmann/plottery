use plottery_lib::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerTreeReference {
    pub num_shapes: usize,
    pub props_inheritable: LayerPropsInheritable,
    pub props: LayerProps,
    pub sublayers: Vec<LayerTreeReference>,

    pub shapes_visible: bool,
    pub sublayers_visible: bool,
}

impl LayerTreeReference {
    pub fn new(layer: &Layer, parent_props_inheritable: &LayerPropsInheritable) -> Self {
        let props_inheritable = parent_props_inheritable.overwrite_with(&layer.props_inheritable);

        let sublayers = layer
            .sublayers
            .iter()
            .map(|sublayer| LayerTreeReference::new(sublayer, &props_inheritable))
            .collect();

        Self {
            num_shapes: layer.len(),
            props_inheritable,
            props: layer.props.clone(),
            sublayers,
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
        let shapes = if self.shapes_visible {
            layer.shapes.clone()
        } else {
            Vec::new()
        };

        let sublayers = if self.sublayers_visible {
            layer
                .iter_sublayers()
                .zip(self.sublayers.iter())
                .filter(|(_sublayer, layer_tree_ref)| layer_tree_ref.sublayers_visible)
                .map(|(sublayer, layer_ref_tree)| {
                    layer_ref_tree.filter_layer_by_visibility(sublayer)
                })
                .collect()
        } else {
            Vec::new()
        };

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(layer.props_inheritable.clone())
            .with_props(layer.props.clone())
    }
}
