#[cfg(test)]
mod test_layer {
    use std::clone;

    use itertools::Itertools;

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

    #[test]
    fn children() {
        let shape = Rect::new(V2::new(0.0, 0.0), V2::new(1.0, 1.0));

        let mut l = Layer::new();
        l.push(shape.clone());
        assert_eq!(l.iter().len(), 1);

        let mut l2 = Layer::new();
        l2.push(shape.clone());
        l2.push(shape.clone());
        assert_eq!(l2.iter().len(), 2);

        let mut l2_2 = Layer::new();
        l2_2.push(shape.clone());
        l2_2.push(shape.clone());
        assert_eq!(l2_2.iter().len(), 2);
        assert_eq!(l2_2.iter_sublayers().len(), 0);
        assert_eq!(l2_2.iter_flattened().collect_vec().len(), 2);

        let mut l3 = Layer::new();
        l3.push(shape.clone());
        l3.push(shape.clone());
        l3.push(shape.clone());
        assert_eq!(l3.iter().len(), 3);
        l2_2.push_layer(l3);
        assert_eq!(l2_2.iter().len(), 2);
        assert_eq!(l2_2.iter_flattened().collect_vec().len(), 5);

        l.push_layer(l2.clone());
        l.push_layer(l2_2.clone());
        assert_eq!(l.iter().len(), 1);
        assert_eq!(l.iter_sublayers().len(), 2);
        assert_eq!(l.iter_flattened().collect_vec().len(), 8);

        let l_clone = l.clone();
        assert_eq!(l_clone.iter().len(), 1);
        assert_eq!(l_clone.iter_sublayers().len(), 2);
        assert_eq!(l_clone.iter_flattened().collect_vec().len(), 8);
    }
}
