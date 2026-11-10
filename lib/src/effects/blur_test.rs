#[cfg(test)]
mod test_blur {
    use std::f32::consts::PI;

    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        effects::blur_noise::{erf_inv, gaussian_noise},
        Blur, Circle, ColorRgb, Layer, Line, Path, Plottable, Rect, Scatter, Shape, V2,
    };

    const PEN: f32 = 0.05;

    fn dots(layer: &Layer) -> Vec<V2> {
        layer
            .iter()
            .filter_map(|shape| match shape {
                Shape::Circle(dot) => Some(dot.center),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn unblurred_line_stays_one_even_path() {
        let line = Path::line(V2::zero(), V2::new(10.0, 0.0));
        let blurred = line.blur(&Blur::new(PEN).with_seed(1));
        let [Shape::Path(path)] = blurred.shapes.as_slice() else {
            panic!("expected one path, got {:?}", blurred.shapes);
        };
        assert_eq!(path.get_start(), Some(&V2::zero()));
        assert!(path.get_end().unwrap().dist(V2::new(10.0, 0.0)) < 1e-4);
        assert!(path.iter().all(|point| point.y == 0.0));
        assert!(path
            .segments()
            .all(|segment| segment.length() <= PEN / 1.9 + 1e-5));
    }

    #[test]
    fn unblurred_closed_shapes_have_no_seam() {
        let blur = Blur::new(PEN).with_seed(1);
        for shape in [
            Circle::new_shape(V2::zero(), 2.0),
            Rect::new_shape(V2::zero(), V2::new(3.0, 1.0)),
        ] {
            let blurred = shape.blur(&blur);
            assert_eq!(blurred.shapes.len(), 1);
            assert!(matches!(&blurred.shapes[0], Shape::Path(path) if path.is_closed()));
        }
    }

    #[test]
    fn seed_decides_the_grain_per_shape() {
        let far = Circle::new_shape(V2::new(100.0, 0.0), 1.0);
        let layer = Layer::new_from([Circle::new_shape(V2::zero(), 1.0), far.clone()]);
        let blur = Blur::new(PEN).with_amount(0.1);
        let blurred = layer.blur(&blur.clone().with_seed(3));
        assert_eq!(
            blurred.shapes,
            layer.blur(&blur.clone().with_seed(3)).shapes
        );
        assert_ne!(
            blurred.shapes,
            layer.blur(&blur.clone().with_seed(4)).shapes
        );

        let edited = Layer::new_from([Circle::new_shape(V2::zero(), 1.5), far]);
        let blurred_edited = edited.blur(&blur.with_seed(3));
        let far_part = |layer: &Layer| -> Vec<Shape> {
            layer
                .shapes
                .iter()
                .filter(|shape| shape.get_points(1.0.into())[0].x > 50.0)
                .cloned()
                .collect()
        };
        assert_eq!(far_part(&blurred), far_part(&blurred_edited));
    }

    #[test]
    fn steps_inside_paths_stay_below_pen_width() {
        let shapes = [
            Circle::new_shape(V2::zero(), 2.0),
            Path::new_shape_from([V2::zero(), V2::new(5.0, 1.0), V2::new(2.0, 4.0)]),
        ];
        for seed in 0..20 {
            for amount in [0.01, 0.05, 0.2] {
                for smoothness in [0.0, 0.3] {
                    let blur = Blur::new(PEN)
                        .with_amount(amount)
                        .with_smoothness(smoothness)
                        .with_seed(seed);
                    for path in shapes.iter().flat_map(|shape| shape.blur(&blur).shapes) {
                        let Shape::Path(path) = path else { continue };
                        assert!(path.len() > 1);
                        assert!(path.segments().all(|segment| segment.length() < PEN));
                    }
                }
            }
        }
    }

    #[test]
    fn every_sample_ends_up_in_a_path_or_dot() {
        // 10 cm at a spacing of 0.5 / 2 cm is 41 samples
        let line = Path::line(V2::zero(), V2::new(10.0, 0.0));
        let (mut total_dots, mut total_paths) = (0, 0);
        for seed in 0..20 {
            let blur = Blur::new(0.5)
                .with_density(2.0)
                .with_amount(0.4)
                .with_seed(seed);
            let blurred = line.blur(&blur);
            let path_points: usize = blurred.paths().map(|path| path.len()).sum();
            assert_eq!(dots(&blurred).len() + path_points, 41);
            total_dots += dots(&blurred).len();
            total_paths += blurred.paths().count();
        }
        assert!(total_dots > 0 && total_paths > 0);
    }

    #[test]
    fn scatter_spreads_as_documented() {
        let mut rng = StdRng::seed_from_u64(1);
        let noise = gaussian_noise(&mut rng, 20_000, 0.01, 0.0, false);
        let mean =
            |f: &dyn Fn(V2) -> f32| noise.iter().map(|g| f(*g)).sum::<f32>() / noise.len() as f32;

        let gaussian = mean(&|g| Scatter::Gaussian.unit_offset(g).length_squared());
        assert!((gaussian - 2.0).abs() < 0.06, "E|d|² = {gaussian}");
        let radial = mean(&|g| Scatter::Radial.unit_offset(g).length());
        assert!((radial - (2.0 / PI).sqrt()).abs() < 0.02, "E|d| = {radial}");
    }

    #[test]
    fn smooth_noise_keeps_its_variance() {
        let mut rng = StdRng::seed_from_u64(2);
        let noise = gaussian_noise(&mut rng, 100_000, 0.01, 0.2, false);
        let variance = noise.iter().map(V2::length_squared).sum::<f32>() / (2 * noise.len()) as f32;
        assert!((variance - 1.0).abs() < 0.1, "variance {variance}");
        let step = noise
            .windows(2)
            .map(|pair| pair[0].dist_squared(pair[1]))
            .sum::<f32>()
            / (noise.len() - 1) as f32;
        assert!(step < 0.5, "mean squared step {step}, white noise has 4");
    }

    #[test]
    fn smooth_noise_wraps_around_closed_shapes() {
        let count = 300;
        let (mut wrap_step, mut inner_step, mut blended_variance) = (0.0, 0.0, 0.0);
        for seed in 0..500 {
            let mut rng = StdRng::seed_from_u64(seed);
            let noise = gaussian_noise(&mut rng, count, 0.01, 0.2, true);
            wrap_step += noise[count - 1].dist_squared(noise[0]);
            inner_step += noise[count / 2].dist_squared(noise[count / 2 + 1]);
            blended_variance += noise[10].length_squared() / 2.0;
        }
        assert!(wrap_step < 2.0 * inner_step, "{wrap_step} vs {inner_step}");
        assert!((blended_variance / 500.0 - 1.0).abs() < 0.2);
    }

    #[test]
    fn erf_inv_matches_known_values() {
        for (x, expected) in [
            (0.0, 0.0),
            (0.5, 0.476_936_276_204_469_9),
            (0.9, 1.163_087_153_676_674_3),
            (0.999, 2.326_753_765_513_524_6),
        ] {
            assert!((erf_inv(x) - expected).abs() < 1e-12, "erf_inv({x})");
            assert_eq!(erf_inv(-x), -erf_inv(x));
        }
        assert!(erf_inv(1.0).is_finite() && erf_inv(1.0) > 5.0);
    }

    #[test]
    fn culling_drops_only_crowded_dots() {
        let layer = Layer::new_from((1..6).map(|i| Circle::new_shape(V2::zero(), i as f32 * 0.3)));
        let min_distance = 0.5 * PEN;
        let blur = Blur::new(PEN).with_amount(0.15).with_seed(5);
        let all = layer.blur(&blur);
        let culled = layer.blur(&blur.with_cull(min_distance));

        assert_eq!(
            all.paths().collect::<Vec<_>>(),
            culled.paths().collect::<Vec<_>>()
        );
        let kept = dots(&culled);
        assert!(kept.len() < dots(&all).len());
        let segments: Vec<Line> = culled.paths().flat_map(Path::segments).collect();
        for (i, dot) in kept.iter().enumerate() {
            assert!(segments
                .iter()
                .all(|segment| segment.closest_point(*dot).dist(*dot) >= min_distance));
            assert!(kept[..i]
                .iter()
                .all(|other| other.dist(*dot) >= min_distance));
        }
    }

    #[test]
    fn layer_blur_keeps_sublayers_and_props() {
        let inner = Layer::new_from([Circle::new_shape(V2::zero(), 1.0)])
            .with_name("inner")
            .with_pen_width_cm(0.1);
        let layer = Layer::new_from_shapes_and_layers(
            vec![Path::new_shape_from([V2::zero(), V2::new(2.0, 0.0)])],
            vec![inner, Layer::new()],
        )
        .with_color(ColorRgb::red());
        let blurred = layer.blur(&Blur::new(PEN).with_amount(0.05).with_seed(1));

        assert_eq!(blurred.props, layer.props);
        assert_eq!(blurred.props_inheritable, layer.props_inheritable);
        assert_eq!(blurred.sublayers.len(), 2);
        for (blurred_sublayer, sublayer) in blurred.sublayers.iter().zip(&layer.sublayers) {
            assert_eq!(blurred_sublayer.props, sublayer.props);
            assert_eq!(
                blurred_sublayer.props_inheritable,
                sublayer.props_inheritable
            );
        }
        assert!(!blurred.shapes.is_empty() && !blurred.sublayers[0].shapes.is_empty());
        assert!(blurred.sublayers[1].is_empty());
    }
}
