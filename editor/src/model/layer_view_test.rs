#[cfg(test)]
mod tests {
    use plottery_lib::{Circle, Layer, Shape, V2};

    use crate::model::layer_view::{
        filter_visible, LayerExpansion, LayerKey, LayerKeySegment, LayerRows, LayerSummary,
        LayerVisibility, RowTarget,
    };

    /// A layer whose only shape is a circle with radius `id`, so shapes can be told apart.
    fn marked(id: u32) -> Layer {
        Layer::new_from([Circle::new_shape(V2::zero(), id as f32)])
    }

    fn group(id: u32, sublayers: Vec<Layer>) -> Layer {
        let mut layer = marked(id);
        layer.sublayers = sublayers;
        layer
    }

    fn key(path: &[&str]) -> LayerKey {
        LayerKey(
            path.iter()
                .map(|name| LayerKeySegment {
                    name: Some(name.to_string()),
                    nth: 0,
                })
                .collect(),
        )
    }

    /// Ids of the shapes that are still drawn.
    fn drawn(layer: &Layer, visibility: &LayerVisibility) -> Vec<u32> {
        filter_visible(layer, visibility)
            .iter()
            .map(|shape| match shape {
                Shape::Circle(circle) => circle.radius as u32,
                _ => panic!("only circles in these tests"),
            })
            .collect()
    }

    #[test]
    fn visibility_of_nested_layers() {
        // 1
        // ├ outlines 2
        // │ ├ details 3
        // │ └ fine 4
        // └ hatching 5
        let layer = group(
            1,
            vec![
                group(
                    2,
                    vec![marked(3).with_name("details"), marked(4).with_name("fine")],
                )
                .with_name("outlines"),
                marked(5).with_name("hatching"),
            ],
        );
        let outlines = RowTarget::Layer(key(&["outlines"]));
        let details = RowTarget::Layer(key(&["outlines", "details"]));
        let mut visibility = LayerVisibility::default();
        assert_eq!(drawn(&layer, &visibility), [1, 2, 3, 4, 5]);

        // A hidden parent hides its subtree, and keeps the flags in it.
        visibility.toggle(&details);
        assert_eq!(drawn(&layer, &visibility), [1, 2, 4, 5]);
        visibility.toggle(&outlines);
        assert_eq!(drawn(&layer, &visibility), [1, 5]);

        // Solo wins over the flags above the row, not over those below it.
        visibility.toggle_solo(details.clone());
        assert_eq!(drawn(&layer, &visibility), [3]);
        visibility.toggle_solo(outlines.clone());
        assert_eq!(drawn(&layer, &visibility), [2, 4]);
        visibility.toggle_solo(RowTarget::OwnShapes(key(&["outlines"])));
        assert_eq!(drawn(&layer, &visibility), [2]);
        visibility.toggle_solo(RowTarget::OwnShapes(LayerKey::default()));
        assert_eq!(drawn(&layer, &visibility), [1]);

        // Leaving the solo brings the flags back as they were.
        visibility.toggle(&RowTarget::OwnShapes(LayerKey::default()));
        assert_eq!(drawn(&layer, &visibility), [1, 5]);
        visibility.toggle(&outlines);
        assert_eq!(drawn(&layer, &visibility), [1, 2, 4, 5]);

        // Own shapes are hidden apart from the sublayers, but only where they have a row.
        visibility.show_all();
        visibility.toggle(&RowTarget::OwnShapes(key(&["outlines"])));
        visibility.toggle(&RowTarget::OwnShapes(LayerKey::default()));
        visibility.toggle(&RowTarget::OwnShapes(key(&["hatching"])));
        assert_eq!(drawn(&layer, &visibility), [3, 4, 5]);
    }

    #[test]
    fn hidden_layers_are_found_again_in_a_changed_tree() {
        let first = Layer::from(vec![
            marked(1).with_name("a"),
            marked(2),
            marked(3).with_name("dup"),
            marked(4).with_name("dup"),
            marked(5),
        ]);
        // Hide "a", the second "dup" and the second unnamed layer, through their rows.
        let rows = LayerRows::new(
            &LayerSummary::new(&first),
            &LayerVisibility::default(),
            &LayerExpansion::default(),
        )
        .rows;
        let mut visibility = LayerVisibility::default();
        for row in [0, 3, 4] {
            visibility.toggle(&rows[row].target);
        }
        assert_eq!(drawn(&first, &visibility), [2, 3]);

        // Reordered, with new layers in between.
        let reordered = Layer::from(vec![
            marked(10),
            marked(11).with_name("new"),
            marked(12).with_name("dup"),
            marked(13).with_name("a"),
            marked(14).with_name("dup"),
            marked(15),
            marked(16).with_name("dup"),
        ]);
        assert_eq!(drawn(&reordered, &visibility), [10, 11, 12, 16]);

        // Flags of layers that are gone hide nothing, and still hold when the layers are back.
        let shrunk = Layer::from(vec![marked(20).with_name("dup"), marked(21)]);
        assert_eq!(drawn(&shrunk, &visibility), [20, 21]);
        assert_eq!(drawn(&first, &visibility), [2, 3]);
    }
}
