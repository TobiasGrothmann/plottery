#[cfg(test)]
mod test_layer {
    use std::collections::HashMap;

    use itertools::Itertools;
    use svg::parser::Event;

    use crate::{
        traits::{normalize::Alignment, Translate},
        Angle, BoundingBox, Circle, Layer, Normalize, Path, Plottable, Rect, Rotate,
        SampleSettings, V2,
    };

    #[test]
    fn iterator() {
        let mut l = Layer::new();

        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
        l.push(Path::new_shape_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 1.0),
        ]));
        l.push(Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0)));

        for shape in l.iter() {
            assert!(!shape.get_points(&SampleSettings::default()).is_empty());
        }

        let l2: Layer = l.clone();
        assert_eq!(l.iter().len(), l2.iter().len());
    }

    #[test]
    fn children() {
        let shape = Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0));

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

        // check len methods
        assert_eq!(l_clone.len(), 1);
        assert_eq!(l_clone.len_sublayers(), 2);
        assert_eq!(l_clone.len_recursive(), 8);
    }

    #[test]
    fn bounding_box() {
        let mut l = Layer::new();

        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
        l.push(Path::new_shape_from(vec![
            V2::new(-0.5, 0.0),
            V2::new(2.0, 2.0),
        ]));
        l.push(Rect::new_shape(V2::new(0.0, 0.0), V2::new(3.0, 1.0)));

        let bounding_box = l.bounding_box().unwrap();
        assert_eq!(bounding_box.bl(), V2::new(-0.5, 0.0));
        assert_eq!(bounding_box.tr(), V2::new(3.0, 2.0));
    }

    #[test]
    fn bounding_box_2() {
        let mut l = Layer::new();

        l.push(Circle::new_shape(V2::new(2.0, 3.0), 1.0));

        let bounding_box = l.bounding_box().unwrap();
        assert_eq!(bounding_box.bl(), V2::new(1.0, 2.0));
        assert_eq!(bounding_box.tr(), V2::new(3.0, 4.0));
    }

    #[test]
    fn svg() {
        let mut l = Layer::new();

        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
        l.push(Path::new_shape_from(vec![
            V2::new(-0.5, 0.0),
            V2::new(2.0, 2.0),
        ]));
        l.push(Rect::new_shape(V2::new(0.0, 0.0), V2::new(3.0, 1.0)));

        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test.svg");
        l.write_svg(svg_path.clone(), 100.0).unwrap();

        // load svg and check that it is valid
        let mut svg_content = String::new();
        let mut paths_count: HashMap<&str, usize> = std::collections::HashMap::new();
        for event in svg::open(svg_path, &mut svg_content).unwrap() {
            match event {
                Event::Tag(path, _type, _attributes) => {
                    if !paths_count.contains_key(path) {
                        paths_count.insert(path, 0);
                    }
                    *paths_count.get_mut(path).unwrap() += 1;
                }
                Event::Error(_) => todo!(),
                Event::Text(_) => todo!(),
                Event::Comment(_) => todo!(),
                Event::Declaration(_) => todo!(),
                Event::Instruction(_) => todo!(),
            }
        }

        assert!(paths_count.contains_key("svg"));

        assert!(paths_count.contains_key("path"));
        assert_eq!(*paths_count.get("path").unwrap(), 1);

        assert!(paths_count.contains_key("circle"));
        assert_eq!(*paths_count.get("circle").unwrap(), 1);

        assert!(paths_count.contains_key("rect"));
        assert_eq!(*paths_count.get("rect").unwrap(), 1);
    }

    #[test]
    fn translate() {
        let mut l = Layer::new();

        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
        l.push(Path::new_shape_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 1.0),
        ]));
        l.push(Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0)));

        let mut sublayer = Layer::new();
        sublayer.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));
        l.push_layer(sublayer);

        let translate_dist = V2::new(2.0, 1.0);

        let l2 = l.translate(&translate_dist);
        assert_eq!(l2.iter().len() + 1, l2.iter_flattened().collect_vec().len());

        let l_box = l.bounding_box().unwrap();
        let l2_box = l2.bounding_box().unwrap();
        println!("{:?}\n{:?}", l_box, l2_box);
        assert_eq!(l_box.bl() + translate_dist, l2_box.bl());
        assert_eq!(l_box.tr() + translate_dist, l2_box.tr());
    }

    #[test]
    fn translate_mut() {
        let mut l = Layer::new();
        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));

        let mut sublayer = Layer::new();
        sublayer.push(Circle::new_shape(V2::new(2.0, 2.0), 1.0));
        l.push_layer(sublayer);

        let translate_dist = V2::new(2.0, 1.0);
        let l_orig = l.clone();
        l.translate_mut(&translate_dist);

        let l_orig_box = l_orig.bounding_box().unwrap();
        let l_box = l.bounding_box().unwrap();
        assert_eq!(l_orig_box.bl() + translate_dist, l_box.bl());
        assert_eq!(l_orig_box.tr() + translate_dist, l_box.tr());
    }

    #[test]
    fn rotate() {
        let mut l = Layer::new();
        l.push(Circle::new_shape(V2::new(1.0, 1.0), 1.0));

        let mut sublayer = Layer::new();
        sublayer.push(Circle::new_shape(V2::new(2.0, 2.0), 1.0));
        l.push_layer(sublayer);

        let pivot = V2::new(3.0, 0.1);
        let mut l2 = l.rotate_around(&pivot, &Angle::from_degrees(55.0));
        l2.rotate_around_mut(&pivot, &Angle::from_degrees(-55.0));

        let l_box = l.bounding_box().unwrap();
        let l2_box = l2.bounding_box().unwrap();
        assert_eq!(l_box.bl(), l2_box.bl());
        assert_eq!(l_box.tr(), l2_box.tr());
    }

    #[test]
    fn normalize() {
        let mut l = Layer::new();
        l.push(Circle::new_shape(V2::new(1.0, 1.0), 3.0));

        l.push_layer(Layer::new_from(vec![Circle::new_shape(
            V2::new(2.0, 2.0),
            4.0,
        )]));

        let l_normalized = l
            .normalize(
                &Rect::new(V2::new(0.5, 0.5), V2::new(1.0, 1.0)),
                Alignment::Center,
            )
            .unwrap();
        let l_normalized_bounds = l_normalized.bounding_box().unwrap();
        assert_eq!(l_normalized_bounds.bl(), V2::new(0.5, 0.5));
        assert_eq!(l_normalized_bounds.tr(), V2::new(1.0, 1.0));
    }
}
