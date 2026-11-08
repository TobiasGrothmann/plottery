#[cfg(test)]
mod test_layer_stats {
    use std::f32::consts::PI;

    use crate::{Circle, Layer, Path, Rect, V2};

    #[test]
    fn lengths_follow_plot_order() {
        let mut layer = Layer::new_from(vec![Path::new_shape_from(vec![
            V2::new(0.0, 3.0),
            V2::new(0.0, 1.0),
        ])]);
        layer.push_layer(Layer::new_from(vec![
            Path::new_shape_from(vec![V2::new(4.0, 3.0), V2::new(4.0, 0.0)]),
            Circle::new_shape(V2::new(8.0, 0.0), 1.0),
        ]));

        let stats = layer.plot_stats();
        assert_eq!(stats.shapes, 3);
        assert_eq!(stats.layers, 2);
        assert!((stats.draw_length - (5.0 + 2.0 * PI)).abs() < 1e-4);
        // enter the first line at its end, the second at its start, the circle at its closest
        // point, then back home: 1 + 4 + 3 + 7
        assert!((stats.travel_length - 15.0).abs() < 1e-4);
        assert_eq!(
            stats.bounds,
            Some(Rect::new(V2::new(0.0, -1.0), V2::new(9.0, 3.0)))
        );
    }

    #[test]
    fn empty_layer() {
        let stats = Layer::new().plot_stats();
        assert_eq!(stats.shapes, 0);
        assert_eq!(stats.travel_length, 0.0);
        assert_eq!(stats.bounds, None);
    }
}
