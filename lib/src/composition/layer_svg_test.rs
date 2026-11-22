#[cfg(test)]
mod test_layer_svg {
    use std::collections::HashMap;

    use svg::parser::Event;

    use crate::{composition::layer_svg::push_mm, Circle, Layer, Path, Rect, Shape, V2};

    #[test]
    fn coordinates_round_to_a_thousandth_without_trailing_zeros() {
        let written = |mm: f64| {
            let mut out = String::new();
            push_mm(&mut out, mm);
            out
        };
        assert_eq!(written(12.3456), "12.346");
        assert_eq!(written(2.5), "2.5");
        assert_eq!(written(100.07), "100.07");
        // zeros of the whole part stay
        assert_eq!(written(10.0), "10");
        // rounding carries into the whole part
        assert_eq!(written(0.9996), "1");
        assert_eq!(written(-0.0501), "-0.05");
        // no negative zero
        assert_eq!(written(-0.0004), "0");
    }

    #[test]
    fn layer_name_is_escaped() {
        let mut l = Layer::new().with_name("a \"b\" & <c>");
        l.push(Path::new_from(vec![V2::new(0.0, 0.0), V2::new(1.0, 1.0)]));

        assert!(l
            .to_svg()
            .contains("<g id=\"a &quot;b&quot; &amp; &lt;c&gt;\""));
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
        l.write_svg(svg_path.clone()).unwrap();

        // load svg and check
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
                Event::Text(_) => {}
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
    fn new_from_svg() {
        let mut l = Layer::new();
        l.push(Path::new_from(vec![
            V2::new(0.0, 0.0),
            V2::new(1.0, 0.0),
            V2::new(1.0, 1.0),
            V2::new(0.0, 0.0),
        ]));
        l.push(Path::new_from(vec![V2::new(2.0, 2.0), V2::new(3.0, 3.0)]));

        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import.svg");
        l.write_svg(svg_path.clone()).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();
        assert_eq!(imported.len(), 2);

        for shape in imported.iter() {
            match shape {
                Shape::Path(path) => assert!(path.as_slice().len() >= 2),
                _ => panic!("Expected imported shape to be Path"),
            }
        }

        let original_paths: Vec<Path> = l
            .iter()
            .map(|shape| match shape {
                Shape::Path(path) => path.clone(),
                _ => panic!("Expected original shape to be Path"),
            })
            .collect();

        let imported_paths: Vec<Path> = imported
            .iter()
            .map(|shape| match shape {
                Shape::Path(path) => path.clone(),
                _ => panic!("Expected imported shape to be Path"),
            })
            .collect();

        assert_eq!(original_paths, imported_paths);
    }

    #[test]
    fn new_from_svg_curves_and_polylines() {
        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import_complex.svg");
        let svg = r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
            <path d='M 0 0 C 10 0, 10 10, 20 10 S 30 20, 40 20 Q 50 20, 60 30 T 70 40 A 5 5 0 0 1 80 50 L 90 50 Z' />
            <line x1='0' y1='90' x2='10' y2='95' />
            <polyline points='20,90 30,95 40,90' />
            <polygon points='50,90 60,95 70,90' />
        </svg>"#;
        std::fs::write(&svg_path, svg).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();

        // one path + one line + one polyline + one polygon
        assert_eq!(imported.len(), 4);

        for shape in imported.iter() {
            match shape {
                Shape::Path(path) => assert!(path.as_slice().len() >= 2),
                _ => panic!("Expected imported shape to be Path"),
            }
        }

        // the curve commands are sampled, not approximated by their end points
        let Shape::Path(curved) = imported.iter().next().unwrap() else {
            panic!("Expected imported shape to be Path");
        };
        assert!(curved.as_slice().len() > 8);
    }

    #[test]
    fn new_from_svg_close_then_relative_line() {
        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import_close_then_line.svg");
        let svg = r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
            <path d='M 10 10 L 20 10 L 20 20 Z l 10 0 l 0 10 z' />
        </svg>"#;
        std::fs::write(&svg_path, svg).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();
        assert_eq!(imported.len(), 2);

        let paths = imported
            .iter()
            .map(|shape| match shape {
                Shape::Path(path) => path.clone(),
                _ => panic!("Expected imported shape to be Path"),
            })
            .collect::<Vec<_>>();

        assert_eq!(paths[0].as_slice().len(), 4);
        assert_eq!(paths[1].as_slice().len(), 4);
    }

    #[test]
    fn new_from_svg_group_transform_matrix() {
        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import_group_transform.svg");
        let svg = r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
            <g transform='matrix(2,0,0,2,10,20)'>
                <path d='m 0 0 l 5 0 l 0 5 z' />
            </g>
        </svg>"#;
        std::fs::write(&svg_path, svg).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();
        assert_eq!(imported.len(), 1);

        let path = match imported.iter().next().unwrap() {
            Shape::Path(path) => path,
            _ => panic!("Expected imported shape to be Path"),
        };

        let points = path.as_slice();
        assert_eq!(points.len(), 4);

        assert_eq!(points[0], V2::new(10.0, 10.0));
        assert_eq!(points[1], V2::new(20.0, 10.0));
        assert_eq!(points[2], V2::new(20.0, 0.0));
        assert_eq!(points[3], V2::new(10.0, 10.0));
    }

    #[test]
    fn new_from_svg_fill_closes_path_without_z() {
        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import_fill_close.svg");
        let svg = r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 100 100'>
            <path style='fill:#000000;stroke-width:0' d='m 10,10 20,0 0,20' />
        </svg>"#;
        std::fs::write(&svg_path, svg).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();
        assert_eq!(imported.len(), 1);

        let path = match imported.iter().next().unwrap() {
            Shape::Path(path) => path,
            _ => panic!("Expected imported shape to be Path"),
        };

        let points = path.as_slice();
        assert_eq!(points.len(), 4);
        assert_eq!(points.first().unwrap(), points.last().unwrap());
    }

    #[test]
    fn new_from_svg_mm_scales_to_cm() {
        let temp_dir = tempfile::tempdir().unwrap();
        let svg_path = temp_dir.path().join("test_import_mm_to_cm.svg");
        let svg = r#"<svg xmlns='http://www.w3.org/2000/svg' width='210mm' height='297mm' viewBox='0 0 210 297'>
            <path d='M 10 10 L 20 10' />
        </svg>"#;
        std::fs::write(&svg_path, svg).unwrap();

        let imported = Layer::new_from_svg(&svg_path).unwrap();
        assert_eq!(imported.len(), 1);

        let path = match imported.iter().next().unwrap() {
            Shape::Path(path) => path,
            _ => panic!("Expected imported shape to be Path"),
        };

        let points = path.as_slice();
        assert_eq!(points.len(), 2);
        assert!((points[0].x - 1.0).abs() < 1e-5);
        assert!((points[1].x - 2.0).abs() < 1e-5);
    }
}
