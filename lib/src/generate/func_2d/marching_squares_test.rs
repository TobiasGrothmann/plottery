#[cfg(test)]
mod test_marching_squares {
    use crate::{MarchingSquares, Plottable, Rect, SampleSettings, V2};

    fn circle_field(radius: f32) -> MarchingSquares {
        let bounds = Rect::new(V2::xy(-2.0), V2::xy(2.0));
        MarchingSquares::new(bounds, SampleSettings::new(10.0), move |p| {
            p.length() - radius
        })
    }

    #[test]
    fn lines_lie_on_the_iso_line() {
        let squares = circle_field(1.0);
        let lines = squares.lines(0.0);
        assert!(lines.len() > 20);
        for line in lines {
            assert!((line.from.length() - 1.0).abs() < 0.1);
            assert!((line.to.length() - 1.0).abs() < 0.1);
        }
    }

    #[test]
    fn paths_join_segments() {
        let squares = circle_field(1.0);
        let paths = squares.paths(0.0);
        assert_eq!(paths.len(), 1);
        let circumference = paths.iter().next().unwrap().length();
        assert!((circumference - 2.0 * std::f32::consts::PI).abs() < 0.2);
    }

    #[test]
    fn captures_state_in_closure() {
        let threshold = 0.5;
        let bounds = Rect::new(V2::zero(), V2::xy(1.0));
        let squares = MarchingSquares::new(bounds, SampleSettings::new(4.0), |p| p.x + threshold);
        assert!(squares
            .lines(1.0)
            .iter()
            .all(|l| (l.from.x - 0.5).abs() < 0.01));
    }

    #[test]
    fn two_circles_give_two_closed_paths() {
        let bounds = Rect::new(V2::new(-3.0, -2.0), V2::new(3.0, 2.0));
        let (a, b) = (V2::new(-1.5, 0.0), V2::new(1.5, 0.0));
        let squares = MarchingSquares::new(bounds, SampleSettings::new(10.0), |p| {
            p.dist(a).min(p.dist(b)) - 1.0
        });
        let paths = squares.paths(0.0);
        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|path| path.is_closed()));
    }

    #[test]
    fn contour_cut_by_bounds_is_one_open_path() {
        let bounds = Rect::new(V2::zero(), V2::xy(2.0));
        let squares = MarchingSquares::new(bounds, SampleSettings::new(10.0), |p| p.length() - 1.0);
        let paths = squares.paths(0.0);
        assert_eq!(paths.len(), 1);
        let path = paths.paths().next().unwrap();
        assert!(!path.is_closed());
        let (start, end) = (path.first().unwrap(), path.last().unwrap());
        assert!(start.x.min(start.y).abs() < 1e-5);
        assert!(end.x.min(end.y).abs() < 1e-5);
    }

    #[test]
    fn samples_on_the_level_keep_paths_whole() {
        let bounds = Rect::new(V2::xy(-2.0), V2::xy(2.0));
        let squares = MarchingSquares::new(bounds, SampleSettings::new(4.0), |p| {
            p.x.abs() + p.y.abs() - 1.0
        });
        let paths = squares.paths(0.0);
        assert_eq!(paths.len(), 1);
        let path = paths.paths().next().unwrap();
        assert!(path.is_closed());
        assert!((path.length() - 4.0 * 2.0_f32.sqrt()).abs() < 1e-4);
    }

    #[test]
    fn saddle_keeps_branches_apart() {
        // 15 cells of 0.4, so the origin is the center of a cell with corners at x * y = ±0.04.
        let bounds = Rect::new(V2::xy(-3.0), V2::xy(3.0));
        let squares = MarchingSquares::new(bounds, SampleSettings::new(2.5), |p| p.x * p.y);
        let paths = squares.paths(0.01);
        assert_eq!(paths.len(), 2);
        for path in paths.paths() {
            assert!(!path.is_closed());
            assert!(path.iter().all(|p| p.x > 0.0) || path.iter().all(|p| p.x < 0.0));
        }
    }
}
