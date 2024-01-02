#[cfg(test)]
mod test_frame {
    use crate::{composition::Frame, Rect, V2};

    #[test]
    fn create() {
        let f = Frame::new(V2::xy(10.0), 1.0);
        assert_eq!(f.inner_rect().size(), V2::xy(8.0));
        assert_eq!(f.outer_rect().size(), V2::xy(10.0));

        let f = Frame::new(V2::xy(10.0), 4.0);
        assert_eq!(f.inner_rect().size(), V2::xy(2.0));
        assert_eq!(f.outer_rect().size(), V2::xy(10.0));

        let f = Frame::new_at(V2::new(3.0, -1.0), V2::xy(10.0), 1.0);
        assert_eq!(f.inner_rect().size(), V2::xy(8.0));
        assert_eq!(f.outer_rect().size(), V2::xy(10.0));

        let f = Frame::new_from_rect(Rect::new(V2::new(2.0, 1.0), V2::new(12.0, 11.0)), 1.0);
        assert_eq!(f.inner_rect().size(), V2::xy(8.0));
        assert_eq!(f.outer_rect().size(), V2::xy(10.0));
    }
}
