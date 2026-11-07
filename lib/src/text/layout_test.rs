#[cfg(test)]
mod test_layout {
    use crate::{
        text::layout::{block_y_offset, break_lines, line_x_offset},
        FontMetrics, TextAlign, VerticalAlign,
    };

    #[test]
    fn breaks_greedily_at_spaces() {
        assert_eq!(
            break_lines(&[3.0, 3.0, 3.0], 1.0, Some(7.5)),
            vec![0..2, 2..3]
        );
    }

    #[test]
    fn over_long_word_gets_its_own_line() {
        assert_eq!(
            break_lines(&[1.0, 9.0, 1.0], 1.0, Some(5.0)),
            vec![0..1, 1..2, 2..3]
        );
    }

    #[test]
    fn no_max_width_is_one_line() {
        assert_eq!(break_lines(&[3.0, 3.0, 3.0], 1.0, None), vec![0..3]);
        assert!(break_lines(&[], 1.0, Some(1.0)).is_empty());
    }

    #[test]
    fn x_offsets() {
        assert_eq!(line_x_offset(TextAlign::Left, 4.0), 0.0);
        assert_eq!(line_x_offset(TextAlign::Center, 4.0), -2.0);
        assert_eq!(line_x_offset(TextAlign::Right, 4.0), -4.0);
    }

    #[test]
    fn y_offsets_of_a_two_line_block() {
        let metrics = FontMetrics {
            ascender: 0.8,
            descender: -0.2,
            line_gap: 0.0,
            cap_height: None,
            x_height: None,
        };
        let offset = |align| block_y_offset(align, &metrics, 1.25, 2);
        assert_eq!(offset(VerticalAlign::Baseline), 0.0);
        assert!((offset(VerticalAlign::Top) + 0.8).abs() < 1e-6);
        assert!((offset(VerticalAlign::Bottom) - 1.45).abs() < 1e-6);
        assert!((offset(VerticalAlign::Center) - 0.325).abs() < 1e-6);
    }
}
