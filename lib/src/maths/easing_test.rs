#[cfg(test)]
mod test_easing {
    use crate::{Easing, EasingKind, EASING_MAX_AMOUNT};

    const AMOUNTS: [f32; 7] = [-1.0, -0.98, -0.5, 0.0, 0.3, 0.98, 1.0];

    fn samples() -> impl Iterator<Item = f32> {
        (0..=100).map(|i| i as f32 / 100.0)
    }

    #[test]
    fn endpoints_are_fixed() {
        for kind in EasingKind::ALL {
            for amount in AMOUNTS {
                let easing = Easing::new(kind, amount);
                assert!(easing.apply(0.0).abs() < 1e-6, "{easing:?}");
                assert!((easing.apply(1.0) - 1.0).abs() < 1e-6, "{easing:?}");
            }
        }
    }

    #[test]
    fn zero_amount_is_identity() {
        for kind in EasingKind::ALL {
            let easing = Easing::new(kind, 0.0);
            for x in samples() {
                assert!((easing.apply(x) - x).abs() < 1e-5, "{kind:?} at {x}");
            }
        }
    }

    #[test]
    fn midpoint_follows_amount() {
        for kind in [EasingKind::Power, EasingKind::Bias] {
            for amount in [-0.9, -0.5, 0.0, 0.25, 0.9] {
                let mid = Easing::new(kind, amount).apply(0.5);
                assert!(
                    (mid - (0.5 + amount / 2.0)).abs() < 1e-5,
                    "{kind:?} {amount}"
                );
            }
        }
        assert!((Easing::s_curve(0.7).apply(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn from_exponent_matches_powf() {
        for p in [0.2, 0.5, 1.0, 2.0, 3.5, 6.0] {
            let easing = Easing::from_exponent(p);
            for x in samples() {
                assert!((easing.apply(x) - x.powf(p)).abs() < 1e-4, "p={p} x={x}");
            }
        }
    }

    #[test]
    fn monotonic_and_finite_at_limits() {
        for kind in EasingKind::ALL {
            for amount in [-EASING_MAX_AMOUNT, EASING_MAX_AMOUNT] {
                let easing = Easing::new(kind, amount);
                let values: Vec<f32> = samples().map(|x| easing.apply(x)).collect();
                assert!(values.iter().all(|v| v.is_finite()), "{easing:?}");
                assert!(values.windows(2).all(|w| w[1] >= w[0]), "{easing:?}");
            }
        }
    }
}
