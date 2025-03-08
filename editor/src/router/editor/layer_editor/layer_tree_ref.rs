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

        let sub_layers = (&layer.sublayers)
            .into_iter()
            .map(|sub_layer| LayerTreeReference::new(&sub_layer, &props))
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
}
