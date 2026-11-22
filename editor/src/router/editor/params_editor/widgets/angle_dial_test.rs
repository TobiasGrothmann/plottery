#[cfg(test)]
mod tests {
    use dioxus::html::geometry::ClientPoint;

    use crate::router::editor::params_editor::widgets::angle_dial::{
        cursor_degrees, dial_value, fit_to_range, stepped_rad, turned,
    };

    /// Direction of a cursor `dx`, `dy` screen pixels away from the dial center.
    fn direction(dx: f64, dy: f64) -> Option<f64> {
        let center = ClientPoint::new(200.0, 100.0);
        cursor_degrees(center, ClientPoint::new(200.0 + dx, 100.0 + dy))
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn up_on_screen_is_a_quarter_turn() {
        let expected = [
            (30.0, 0.0, 0.0),
            (0.0, -30.0, 90.0),
            (-30.0, 0.0, 180.0),
            (0.0, 30.0, 270.0),
        ];
        for (dx, dy, degrees) in expected {
            assert!(
                direction(dx, dy).is_some_and(|d| close(d, degrees)),
                "{dx} {dy}"
            );
        }
        assert_eq!(direction(1.0, -1.0), None);
    }

    #[test]
    fn rounding_up_to_a_full_turn_gives_zero() {
        assert_eq!(stepped_rad(359.6, 1.0), 0.0);
        assert_eq!(stepped_rad(355.0, 15.0), 0.0);
        assert!(close(stepped_rad(50.3, 1.0).to_degrees() as f64, 50.0));
        assert!(close(stepped_rad(50.3, 15.0).to_degrees() as f64, 45.0));
    }

    #[test]
    fn turning_across_zero_takes_the_short_way() {
        assert!(close(turned(350.0, 350.0, 10.0, 1.0), 10.0));
        assert!(close(turned(10.0, 10.0, 350.0, 1.0), 350.0));
        // a quarter of 20° forwards and backwards
        assert!(close(turned(358.0, 350.0, 10.0, 0.25), 3.0));
        assert!(close(turned(2.0, 10.0, 350.0, 0.25), 357.0));
    }

    fn deg(degrees: f32) -> f32 {
        degrees.to_radians()
    }

    #[test]
    fn range_across_zero_keeps_negative_angles() {
        let range = (deg(-45.0), deg(45.0));
        let (rad, stopped) = fit_to_range(deg(350.0), range);
        assert!(close(rad.to_degrees() as f64, -10.0) && !stopped);
        let (rad, stopped) = fit_to_range(deg(30.0), range);
        assert!(close(rad.to_degrees() as f64, 30.0) && !stopped);
    }

    #[test]
    fn outside_the_range_gives_the_closer_bound() {
        let range = (deg(0.0), deg(90.0));
        assert_eq!(fit_to_range(deg(100.0), range), (deg(90.0), true));
        assert_eq!(fit_to_range(deg(350.0), range), (deg(0.0), true));
    }

    #[test]
    fn bound_stops_the_handle() {
        let range = Some((deg(0.0), deg(90.0)));
        let (rad, handle) = dial_value(120.0, 1.0, range);
        assert_eq!(rad, deg(90.0));
        assert!(close(handle, 90.0));
        // inside, the handle keeps its fraction for slow turning
        let (rad, handle) = dial_value(45.3, 1.0, range);
        assert!(close(rad.to_degrees() as f64, 45.0));
        assert!(close(handle, 45.3));
    }
}
