#[cfg(test)]
mod tests {
    use crate::maths::get_corner_sharpness;
    use plottery_lib::{geometry::TransformMatrix, Angle, V2};

    #[test]
    fn corner_sharpness_180() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(0.0, 0.0);

        test_with_rotation_and_offset(&a, &b, &c, 1.0);
    }

    #[test]
    fn corner_sharpness_90() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(1.0, 1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.5);
    }

    #[test]
    fn corner_sharpness_neg90() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(1.0, -1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.5);
    }

    #[test]
    fn corner_sharpness_45() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(2.0, 1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.25);
    }

    #[test]
    fn corner_sharpness_neg45() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(2.0, -1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.25);
    }

    #[test]
    fn corner_sharpness_135() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(0.0, 1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.75);
    }

    #[test]
    fn corner_sharpness_neg135() {
        let a = V2::new(0.0, 0.0);
        let b = V2::new(1.0, 0.0);
        let c = V2::new(0.0, -1.0);

        test_with_rotation_and_offset(&a, &b, &c, 0.75);
    }

    fn test_with_rotation_and_offset(a: &V2, b: &V2, c: &V2, expected_sharpness: f32) {
        let angle_steps = 100;
        for angle_step in 0..(angle_steps + 1) {
            let angle = Angle::from_rotations(angle_step as f32 / angle_steps as f32);

            for x in -6..6 {
                for y in -6..6 {
                    let transform = TransformMatrix::builder()
                        .rotate(&angle)
                        .translate(&V2::new(x as f32 / 5.0, y as f32 / 5.0))
                        .build();

                    let at = transform.mul_vector(a);
                    let bt = transform.mul_vector(b);
                    let ct = transform.mul_vector(c);

                    let sharpness = get_corner_sharpness(&at, &bt, &ct);
                    assert!(sharpness <= 1.0);
                    assert!(sharpness >= 0.0);
                    assert!((sharpness - expected_sharpness).abs() <= 0.001);
                }
            }
        }
    }
}
