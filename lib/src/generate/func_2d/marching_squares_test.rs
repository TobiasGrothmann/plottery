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
}
