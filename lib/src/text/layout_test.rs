#[cfg(test)]
mod test_layout {
    use crate::{
        text::layout::{
            block_y_offset, break_lines, break_lines_balanced, line_width, line_x_offset,
        },
        FontMetrics, TextAlign, VerticalAlign,
    };

    #[test]
    fn breaks_greedily_at_spaces() {
        let tokens = [(3.0, 1.0), (3.0, 1.0), (3.0, 1.0)];
        assert_eq!(break_lines(&tokens, Some(7.5)), vec![0..2, 2..3]);
    }

    #[test]
    fn over_long_word_gets_its_own_line() {
        let tokens = [(1.0, 1.0), (9.0, 1.0), (1.0, 1.0)];
        assert_eq!(break_lines(&tokens, Some(5.0)), vec![0..1, 1..2, 2..3]);
    }

    #[test]
    fn no_max_width_is_one_line() {
        let tokens = [(3.0, 1.0), (3.0, 1.0), (3.0, 1.0)];
        assert_eq!(break_lines(&tokens, None), vec![0..3]);
        assert!(break_lines(&[], Some(1.0)).is_empty());
    }

    #[test]
    fn zero_glue_after_a_hyphen_lets_the_part_fit() {
        let hyphenated = [(3.0, 0.0), (3.0, 1.0), (3.0, 1.0)];
        assert_eq!(break_lines(&hyphenated, Some(6.5)), vec![0..2, 2..3]);
        let spaced = [(3.0, 1.0), (3.0, 1.0), (3.0, 1.0)];
        assert_eq!(break_lines(&spaced, Some(6.5)), vec![0..1, 1..2, 2..3]);
        assert!((line_width(&hyphenated, 0..2) - 6.0).abs() < 1e-6);
        assert!((line_width(&spaced, 0..3) - 11.0).abs() < 1e-6);
    }

    #[test]
    fn balanced_breaks_are_less_ragged_than_greedy() {
        let tokens = [(2.0, 1.0), (2.0, 1.0), (2.0, 1.0), (2.0, 1.0), (1.0, 1.0)];
        assert_eq!(break_lines(&tokens, Some(8.0)), vec![0..3, 3..5]);
        assert_eq!(break_lines_balanced(&tokens, 8.0), vec![0..2, 2..5]);
    }

    #[test]
    fn x_offsets() {
        assert_eq!(line_x_offset(TextAlign::Left, 4.0), 0.0);
        assert_eq!(line_x_offset(TextAlign::Justify, 4.0), 0.0);
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
