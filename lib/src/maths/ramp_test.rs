#[cfg(test)]
mod test_ramp {
    use crate::{Easing, Ramp};

    #[test]
    fn falling_ramp_keeps_its_direction() {
        let ramp = Ramp::new(5.0, 1.0);
        assert_eq!(ramp.at(0.0), 5.0);
        assert_eq!(ramp.at(1.0), 1.0);
        assert_eq!(ramp.at(0.25), 4.0);
        assert_eq!(ramp.at(2.0), 1.0);
    }

    #[test]
    fn easing_bends_the_middle_not_the_ends() {
        let ramp = Ramp::new(10.0, 20.0).with_easing(Easing::from_exponent(2.0));
        assert_eq!(ramp.at(0.0), 10.0);
        assert_eq!(ramp.at(1.0), 20.0);
        assert!((ramp.at(0.5) - 12.5).abs() < 1e-4);
    }

    #[test]
    fn steps_include_both_ends() {
        let ramp = Ramp::new(0.0, 3.0);
        assert_eq!(ramp.steps(0), Vec::<f32>::new());
        assert_eq!(ramp.steps(1), vec![0.0]);
        assert_eq!(ramp.steps(4), vec![0.0, 1.0, 2.0, 3.0]);
    }
}
