#[cfg(test)]
mod test_matrix {
    use crate::{geometry::TransformMatrix, Angle, V2};

    #[test]
    fn multiply_matrix_0() {
        let a = TransformMatrix {
            tl: 1.0,
            bl: 2.0,
            tr: 2.0,
            br: 3.0,
            u: 1.0,
            v: 2.0,
        };

        let b = TransformMatrix {
            tl: 1.0,
            bl: 1.0,
            tr: 4.0,
            br: 2.0,
            u: 2.0,
            v: 1.0,
        };

        let c = TransformMatrix {
            tl: 3.0,
            bl: 5.0,
            tr: 8.0,
            br: 14.0,
            u: 5.0,
            v: 9.0,
        };
        assert_eq!(c, a.mul_matrix(&b));
    }

    #[test]
    fn multiply_matrix_1() {
        let a = TransformMatrix {
            tl: 1.0,
            bl: 4.0,
            tr: 2.0,
            br: 5.0,
            u: 3.0,
            v: 6.0,
        };

        let b = TransformMatrix {
            tl: 5.0,
            bl: 8.0,
            tr: 6.0,
            br: 9.0,
            u: 7.0,
            v: 10.0,
        };

        let c = TransformMatrix {
            tl: 21.0,
            bl: 60.0,
            tr: 24.0,
            br: 69.0,
            u: 30.0,
            v: 84.0,
        };
        assert_eq!(c, a.mul_matrix(&b));
        assert_ne!(c, b.mul_matrix(&c)); // the order matters here
    }

    #[test]
    fn multiply_vector() {
        let a = TransformMatrix {
            tl: 1.0,
            bl: 4.0,
            tr: 2.0,
            br: 5.0,
            u: 3.0,
            v: 6.0,
        };

        let v = V2 { x: 2.0, y: 1.0 };
        let r = a.mul_vector(&v);

        assert_eq!(r, V2::new(7.0, 19.0));
    }

    #[test]
    fn translate() {
        let t = TransformMatrix::translate(&V2::new(2.0, 3.0));
        let v = V2 { x: 0.5, y: 0.5 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(2.5, 3.5));
    }

    #[test]
    fn mirror_x() {
        let t = TransformMatrix::mirror_x();
        let v = V2 { x: 1.0, y: 1.0 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(1.0, -1.0));
    }

    #[test]
    fn mirror_y() {
        let t = TransformMatrix::mirror_y();
        let v = V2 { x: 1.0, y: 1.0 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(-1.0, 1.0));
    }

    #[test]
    fn scale2d() {
        let t = TransformMatrix::scale_2d(&V2::new(2.0, 3.0));
        let v = V2 { x: 1.0, y: 1.0 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(2.0, 3.0));
    }

    #[test]
    fn rotate() {
        let t = TransformMatrix::rotate(&Angle::from_degrees(90.0));
        let v = V2 { x: 1.0, y: 0.0 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(0.0, 1.0));
    }

    #[test]
    fn shear() {
        let t = TransformMatrix::shear(&V2::new(1.0, 0.0));
        let v = V2 { x: 1.0, y: 1.0 };
        let r = t.mul_vector(&v);

        assert_eq!(r, V2::new(2.0, 1.0));
    }

    #[test]
    fn combine_transforms_0() {
        let scale = TransformMatrix::scale_2d(&V2::xy(2.0));
        let translate = TransformMatrix::translate(&V2::new(1.0, 0.0));

        let combined = translate.mul_matrix(&scale); // first scale, then translate

        let v = V2 { x: 0.0, y: 0.5 };
        let r = combined.mul_vector(&v);

        println!("{:?}", r);
        assert_eq!(r, V2::new(1.0, 1.0));
    }

    #[test]
    fn combine_transforms_1() {
        let scale = TransformMatrix::scale_2d(&V2::xy(2.0));
        let translate = TransformMatrix::translate(&V2::new(1.0, 0.0));

        let combined = TransformMatrix::combine_transforms(&vec![scale, translate]); // first scale, then translate

        let v = V2 { x: 0.0, y: 0.5 };
        let r = combined.mul_vector(&v);

        println!("{:?}", r);
        assert_eq!(r, V2::new(1.0, 1.0));
    }

    #[test]
    fn combine_to_result_in_identity() {
        let scale = TransformMatrix::scale_2d(&V2::xy(2.0));
        let translate = TransformMatrix::translate(&V2::new(1.0, 0.0));
        let scale_back = TransformMatrix::scale_2d(&V2::xy(0.5));
        let translate_back = TransformMatrix::translate(&V2::new(-1.0, 0.0));

        let transforms = vec![scale, translate, translate_back, scale_back];
        let combined = TransformMatrix::combine_transforms(&transforms);

        let v = V2 { x: 1.0, y: 1.0 };
        let r = combined.mul_vector(&v);

        assert_eq!(r, v);
    }

    #[test]
    fn builder() {
        let matrix = TransformMatrix::builder()
            .scale(2.0)
            .translate(&V2::new(2.0, 1.0))
            .translate(&V2::new(-3.0, 1.0))
            .translate(&V2::new(1.0, -2.0))
            .scale(0.5)
            .build();

        assert_eq!(matrix, TransformMatrix::identity());
    }
}
