#[cfg(test)]
mod test_hatch {
    use crate::{Angle, Circle, Hatch, Layer, Path, Plottable, Rect, Rotate, Shape, V2};

    const LINE_DIST: f32 = 0.1;
    /// Exact in binary, so grid-snapped shapes land exactly on lines.
    const GRID: f32 = 0.25;

    fn square(min: f32, max: f32) -> Shape {
        Rect::new_shape(V2::new(min, min), V2::new(max, max))
    }

    /// Base of `2n - 1` × 1 with `n` teeth of width 1 rising to `top`, scaled by `unit`.
    fn comb(n: usize, top: f32, unit: f32) -> Path {
        let width = (2 * n - 1) as f32;
        let teeth = (0..n).rev().flat_map(|i| {
            let x = 2.0 * i as f32;
            [(x + 1.0, 1.0), (x + 1.0, top), (x, top), (x, 1.0)]
        });
        let points: Vec<V2> = [(0.0, 0.0), (width, 0.0)]
            .into_iter()
            .chain(teeth.skip(1))
            .map(|(x, y)| V2::new(x, y) * unit)
            .collect();
        Path::new_from(points).closed()
    }

    fn paths(layer: &Layer) -> Vec<&Path> {
        layer.paths().collect()
    }

    fn pen_downs(layer: &Layer) -> usize {
        assert_eq!(layer.len(), paths(layer).len(), "only paths expected");
        layer.len()
    }

    #[test]
    fn lift_counts() {
        let hatch = Hatch::new(LINE_DIST);
        assert_eq!(pen_downs(&square(0.0, 2.0).hatch(&hatch)), 1);
        assert_eq!(pen_downs(&Circle::new(V2::zero(), 1.0).hatch(&hatch)), 1);

        let donut = Layer::new_from([
            Circle::new_shape(V2::zero(), 2.0),
            Circle::new_shape(V2::zero(), 1.0),
        ]);
        assert_eq!(pen_downs(&donut.hatch(&hatch)), 2);

        for n in 1..=5 {
            assert_eq!(pen_downs(&comb(n, 3.0, 1.0).hatch(&hatch)), n, "comb {n}");
        }

        let open_line = Path::new_shape_from([V2::new(-1.0, -1.0), V2::new(7.0, 7.0)]);
        let nested = Layer::new_from([
            square(0.0, 6.0),
            square(1.0, 5.0),
            square(2.0, 4.0),
            open_line,
        ]);
        assert_eq!(pen_downs(&nested.hatch(&hatch)), 3);
    }

    /// Plain lines of `shape` back in the hatch frame, checked for degenerate output.
    fn checked_lines(shape: &Shape, angle: Angle) -> Vec<(V2, V2)> {
        let hatch = Hatch::new(GRID).with_angle(angle).with_offset(0.0);
        let connected = shape.hatch(&hatch);
        for path in paths(&connected) {
            let points = path.get_points_ref();
            assert!(points.len() >= 2);
            assert!(points.windows(2).all(|w| w[0].dist(w[1]) > 1e-6));
        }

        let to_frame = Angle::from_rad(-angle.to_rad());
        let lines: Vec<(V2, V2)> = paths(&shape.hatch(&hatch.with_connected(false)))
            .iter()
            .map(|path| {
                let [a, b] = path.get_points_ref()[..] else {
                    panic!("plain line with {} points", path.get_points_ref().len());
                };
                (a.rotate(to_frame), b.rotate(to_frame))
            })
            .collect();
        for (i, &(a, b)) in lines.iter().enumerate() {
            assert!(a.dist(b) > 1e-6, "zero-length line {a:?}");
            // an odd crossing count on a line would pair crossings of two lines
            assert!((a.y - b.y).abs() < 1e-4, "line spans two lines: {a:?} {b:?}");
            let duplicate = lines[..i]
                .iter()
                .any(|&(c, d)| c.dist(a) < 1e-4 && d.dist(b) < 1e-4);
            assert!(!duplicate, "duplicate line {a:?} {b:?}");
            // lines may lie on an edge, so look just beside them
            let mid = (a + b) * 0.5;
            let inside = [-1e-3, 1e-3]
                .iter()
                .any(|&dy| shape.contains_point((mid + V2::new(0.0, dy)).rotate(angle)));
            assert!(inside, "line outside {mid:?}");
        }
        lines
    }

    #[test]
    fn grid_snapped_edges_and_corners() {
        let comb = Shape::Path(comb(3, 3.0, 2.0 * GRID));
        for degrees in [0.0, 90.0] {
            let angle = Angle::from_degrees(degrees);
            // lines at 0.25 .. 2.0, the bottom edge's line belongs to the band below.
            // Rotating by 90° moves the edges by float noise, to either side of their lines.
            let count = checked_lines(&square(0.0, 2.0), angle).len();
            let expected = if degrees == 0.0 { 8..=8 } else { 7..=9 };
            assert!(expected.contains(&count), "{count} lines at {degrees}°");
            assert!(!checked_lines(&comb, angle).is_empty());
        }
    }

    #[test]
    fn plain_lines_run_at_the_angle() {
        let angle = Angle::from_degrees(30.0);
        let hatch = Hatch::new(LINE_DIST).with_angle(angle).with_connected(false);
        let layer = Circle::new(V2::new(1.0, 2.0), 1.5).hatch(&hatch);
        assert!(layer.len() > 20);
        for path in paths(&layer) {
            let [a, b] = path.get_points_ref()[..] else {
                panic!("plain line with {} points", path.get_points_ref().len());
            };
            let degrees = (b - a).angle().to_degree().rem_euclid(180.0);
            assert!((degrees - 30.0).abs() < 0.01, "line at {degrees}°");
        }
    }

    #[test]
    fn separate_shapes_share_the_grid() {
        let angle = Angle::from_degrees(30.0);
        let offset = 0.3;
        let hatch = Hatch::new(LINE_DIST)
            .with_angle(angle)
            .with_offset(offset)
            .with_connected(false);
        let normal = V2::new(-angle.to_rad().sin(), angle.to_rad().cos());
        for shape in [
            Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0)),
            Rect::new_shape(V2::new(13.7, 5.2), V2::new(15.1, 6.9)),
        ] {
            let layer = shape.hatch(&hatch);
            assert!(!layer.is_empty());
            for path in paths(&layer) {
                let phase = (path.get_points_ref()[0].dot(normal) / LINE_DIST - offset)
                    .rem_euclid(1.0);
                let off_grid = phase.min(1.0 - phase);
                assert!(off_grid < 1e-2, "line {off_grid} line distances off the grid");
            }
        }
    }
}
