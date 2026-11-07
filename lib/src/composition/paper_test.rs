#[cfg(test)]
mod test_paper {
    use crate::{Margin, Paper, PaperSize, V2};

    #[test]
    fn landscape_swaps_axes() {
        let paper = Paper::din_a(4).with_landscape(true);
        assert_eq!(paper.size(), V2::new(29.7, 21.0));
        assert_eq!(
            Paper::custom(V2::new(30.0, 10.0)).size(),
            V2::new(10.0, 30.0)
        );
    }

    #[test]
    fn margin_fraction_uses_short_side() {
        let paper = Paper::din_a(4).with_margin(Margin::Fraction(0.1));
        assert!((paper.margin_cm() - 2.1).abs() < 1e-5);
        assert!((paper.with_landscape(true).margin_cm() - 2.1).abs() < 1e-5);
        assert_eq!(paper.with_margin(Margin::Cm(1.5)).margin_cm(), 1.5);
    }

    #[test]
    fn frame_matches_size_and_margin() {
        let paper = Paper::new(
            PaperSize::Custom(V2::new(10.0, 20.0)),
            false,
            Margin::Cm(1.0),
        );
        let frame = paper.frame();
        assert_eq!(frame.outer_rect().size(), V2::new(10.0, 20.0));
        assert_eq!(frame.inner_rect().size(), V2::new(8.0, 18.0));
        assert_eq!(paper.rect().size(), V2::new(10.0, 20.0));
    }
}
