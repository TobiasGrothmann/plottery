#[cfg(test)]
mod test_layer {
    use std::collections::HashMap;

    use itertools::Itertools;
    use rand::{seq::SliceRandom, SeedableRng};
    use svg::parser::Event;

    use crate::{
        geometry::ToAngle,
        traits::{normalize::Alignment, Translate},
        Angle, BoundingBox, Circle, FloatInterpolation, Layer, Normalize, Path, Plottable, Rect,
        Rotate, SampleSettings, LARGE_EPSILON, V2,
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

    #[test]
    fn combine_shapes_end_to_start() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::xy(0.0), V2::xy(1.0)]));
        a.push_path(Path::new_from(vec![V2::xy(1.0), V2::xy(2.0)]));
        let b = a.combine_shapes_flat(Some(Angle::from_degrees(1.0)));
        println!("{:?}", b);
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn combine_shapes_end_to_end() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::xy(0.0), V2::xy(1.0)]));
        a.push_path(Path::new_from(vec![V2::xy(2.0), V2::xy(1.0)]));
        let b = a.combine_shapes_flat(Some(Angle::from_degrees(1.0)));
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn combine_shapes_start_to_start() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::xy(1.0), V2::xy(0.0)]));
        a.push_path(Path::new_from(vec![V2::xy(1.0), V2::xy(2.0)]));
        let b = a.combine_shapes_flat(Some(Angle::from_degrees(1.0)));
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn combine_shapes_start_to_end() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::xy(1.0), V2::xy(0.0)]));
        a.push_path(Path::new_from(vec![V2::xy(2.0), V2::xy(1.0)]));
        let b = a.combine_shapes_flat(Some(Angle::from_degrees(1.0)));
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn combine_shapes_rect() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(0.0, 1.0)]));
        a.push_path(Path::new_from(vec![V2::new(1.0, 1.0), V2::new(1.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(1.0, 1.0), V2::new(0.0, 1.0)]));

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);

        let c = a.combine_shapes_flat(Some(Angle::from_degrees(1.0)));
        assert_eq!(c.len(), 4);
    }

    #[test]
    fn combine_shapes_angle_end_to_start() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(1.0, 0.0), V2::new(2.0, 1.0)])); // 45°

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);

        let c = a.combine_shapes_flat(Some(Angle::from_degrees(40.0)));
        assert_eq!(c.len(), 2);

        let d = a.combine_shapes_flat(Some(Angle::from_degrees(50.0)));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn combine_shapes_angle_end_to_end() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(2.0, 1.0), V2::new(1.0, 0.0)])); // 45°

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);

        let c = a.combine_shapes_flat(Some(Angle::from_degrees(40.0)));
        assert_eq!(c.len(), 2);

        let d = a.combine_shapes_flat(Some(Angle::from_degrees(50.0)));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn combine_shapes_angle_start_to_end() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::new(1.0, 0.0), V2::new(0.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(2.0, 1.0), V2::new(1.0, 0.0)])); // 45°

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);

        let c = a.combine_shapes_flat(Some(Angle::from_degrees(40.0)));
        assert_eq!(c.len(), 2);

        let d = a.combine_shapes_flat(Some(Angle::from_degrees(50.0)));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn combine_shapes_angle_start_to_start() {
        let mut a = Layer::new();
        a.push_path(Path::new_from(vec![V2::new(1.0, 0.0), V2::new(0.0, 0.0)]));
        a.push_path(Path::new_from(vec![V2::new(1.0, 0.0), V2::new(2.0, 1.0)])); // 45°

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);

        let c = a.combine_shapes_flat(Some(Angle::from_degrees(40.0)));
        assert_eq!(c.len(), 2);

        let d = a.combine_shapes_flat(Some(Angle::from_degrees(50.0)));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn combine_shapes_angles() {
        let center = V2::new(3.5, 5.0);

        for angle_diff_deg in (-85.0).lerp_iter_fixed(85.0, 20) {
            let angle_diff = angle_diff_deg.degrees();

            for start_angle_deg in 0.0.lerp_iter_fixed(360.0, 20) {
                let start_angle = start_angle_deg.degrees();
                let end_angle = start_angle + angle_diff;

                let point_before = center - V2::polar(start_angle, 1.0);
                let point_after = center + V2::polar(end_angle, 1.0);

                let a = Layer::new_from(vec![
                    Path::new_shape_from(vec![point_before, center]),
                    Path::new_shape_from(vec![center, point_after]),
                ]);
                let b = Layer::new_from(vec![
                    Path::new_shape_from(vec![center, point_before]),
                    Path::new_shape_from(vec![center, point_after]),
                ]);
                let c = Layer::new_from(vec![
                    Path::new_shape_from(vec![point_before, center]),
                    Path::new_shape_from(vec![point_after, center]),
                ]);
                let d = Layer::new_from(vec![
                    Path::new_shape_from(vec![center, point_before]),
                    Path::new_shape_from(vec![point_after, center]),
                ]);

                let a2 = Layer::new_from(vec![
                    Path::new_shape_from(vec![center, point_after]),
                    Path::new_shape_from(vec![point_before, center]),
                ]);
                let b2 = Layer::new_from(vec![
                    Path::new_shape_from(vec![center, point_after]),
                    Path::new_shape_from(vec![center, point_before]),
                ]);
                let c2 = Layer::new_from(vec![
                    Path::new_shape_from(vec![point_after, center]),
                    Path::new_shape_from(vec![point_before, center]),
                ]);
                let d2 = Layer::new_from(vec![
                    Path::new_shape_from(vec![point_after, center]),
                    Path::new_shape_from(vec![center, point_before]),
                ]);

                for layer in [a, b, c, d, a2, b2, c2, d2].iter() {
                    let combined = layer.combine_shapes_flat(None);
                    assert_eq!(combined.len(), 1);

                    let combined =
                        layer.combine_shapes_flat(Some(angle_diff.abs() + 2.0.degrees()));
                    assert_eq!(combined.len(), 1);

                    let combined =
                        layer.combine_shapes_flat(Some(angle_diff.abs() - 2.0.degrees()));
                    assert_eq!(combined.len(), 2);
                }
            }
        }
    }

    #[test]
    fn combine_shapes_three() {
        let center = V2::new(0.0, 0.0);
        let angle_a = 180.degrees();
        let angle_b = 90.degrees();
        let angle_c = 0.degrees();

        let a = Path::new_shape_from(vec![center + V2::polar(angle_a, 1.0), center]);
        let b = Path::new_shape_from(vec![center + V2::polar(angle_b, 1.0), center]);
        let c = Path::new_shape_from(vec![center + V2::polar(angle_c, 1.0), center]);

        let layer_a = Layer::new_from(vec![a.clone(), b.clone(), c.clone()]);
        let layer_b = Layer::new_from(vec![a.clone(), c.clone(), b.clone()]);
        let layer_c = Layer::new_from(vec![b.clone(), a.clone(), c.clone()]);
        let layer_d = Layer::new_from(vec![b.clone(), c.clone(), a.clone()]);
        let layer_e = Layer::new_from(vec![c.clone(), a.clone(), b.clone()]);
        let layer_f = Layer::new_from(vec![c.clone(), b.clone(), a.clone()]);

        for layer in [layer_a, layer_b, layer_c, layer_d, layer_e, layer_f].iter() {
            let combined = layer.combine_shapes_flat(Some(10.degrees()));
            assert_eq!(combined.len(), 2);

            let shapes: Vec<_> = combined
                .iter_flattened()
                .sorted_by_key(|a| a.length().round() as i32) // sort by length ascending
                .collect();

            println!("{:?}", shapes);

            assert!(shapes[0].get_points(&SampleSettings::default())[0] == V2::new(0.0, 1.0));
            assert!(shapes[0].get_points(&SampleSettings::default())[1] == V2::new(0.0, 0.0));

            assert!(shapes[1].get_points(&SampleSettings::default())[0].y <= LARGE_EPSILON);
            assert!(shapes[1].get_points(&SampleSettings::default())[1] == V2::new(0.0, 0.0));
            assert!(shapes[1].get_points(&SampleSettings::default())[2].y <= LARGE_EPSILON);
        }
    }

    #[test]
    fn combine_shapes_star() {
        let num_lines = 10;
        let angle_per_line = Angle::from_rotations(1.0 / num_lines as f32);

        let mut a = Layer::new();
        for i in 0..num_lines {
            let path = Path::new_from(vec![V2::xy(0.0), V2::polar(angle_per_line * i as f32, 1.0)]);
            a.push_path(path);
        }

        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), num_lines / 2);
    }

    #[test]
    fn combine_shapes_long_line() {
        let mut segments = (0..100)
            .map(|i| Path::new_from(vec![V2::new(i as f32, 0.0), V2::new((i + 1) as f32, 0.0)]))
            .enumerate()
            .map(|(i, path)| if i % 3 == 0 { path.reverse() } else { path })
            .map(|path| path.to_shape())
            .collect_vec();

        // create a new rng with a fixed seed
        let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
        segments.shuffle(&mut rng);

        let a = Layer::new_from(segments);
        let b = a.combine_shapes_flat(None);
        assert_eq!(b.len(), 1);
    }

    #[test]
    fn optimize() {
        let l = Layer::new_from(vec![
            Path::new_shape_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 0.0)]),
            Path::new_shape_from(vec![V2::new(10.0, 0.0), V2::new(9.0, 0.0)]),
            Path::new_shape_from(vec![V2::new(4.0, 0.0), V2::new(8.0, 0.0)]),
        ]);
        let o = l.optimize();
        let o2 = l.optimize_recursive();

        assert_eq!(o.len(), l.len());
        assert_eq!(o.len_recursive(), l.len_recursive());
        assert_eq!(l.shapes[0], o.shapes[0]);
        assert_eq!(l.shapes[1], o.shapes[2]);
        assert_eq!(l.shapes[2], o.shapes[1]);

        assert_eq!(o2.len(), l.len());
        assert_eq!(o2.len_recursive(), l.len_recursive());
        assert_eq!(l.shapes[0], o2.shapes[0]);
        assert_eq!(l.shapes[1], o2.shapes[2]);
        assert_eq!(l.shapes[2], o2.shapes[1]);
    }
}
