#[cfg(test)]
mod test_layer {
    use crate::{layer::Layer, Circle, Path, Rect, SampleSettings, V2};

    #[test]
    fn iterator() {
        let mut l = Layer::new();

        l.push(Circle::new(V2::new(1.0, 1.0), 1.0));
        l.push(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 1.0)]));
        l.push(Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0)));

        for shape in l.iter() {
            assert!(shape.get_points(&SampleSettings::default()).len() > 0);
        }

        let l2: Layer = l.clone();
        assert_eq!(l.iter().len(), l2.iter().len());
    }
}
