#[cfg(test)]
mod test_interpolator {
    use crate::{Angle, FloatInterpolation, SampleSettings, V2};

    #[test]
    fn zero_steps_yield_only_the_start() {
        let p = V2::new(1.0, 2.0);
        assert_eq!(
            p.lerp_iter(p, SampleSettings::default())
                .collect::<Vec<_>>(),
            [p]
        );
        let a = Angle::from_degrees(30.0);
        let angles: Vec<_> = a.lerp_iter(a, SampleSettings::default(), 1.0).collect();
        assert_eq!(angles, [a]);
        assert_eq!(2.0_f32.lerp_iter(5.0, 0.0).collect::<Vec<_>>(), [2.0]);
    }
}
