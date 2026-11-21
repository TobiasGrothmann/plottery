#[cfg(test)]
mod tests {
    use dioxus::html::geometry::ClientPoint;

    use crate::router::editor::params_editor::widgets::angle_dial::{
        cursor_degrees, stepped_rad, turned,
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
}
