#[cfg(test)]
mod test_text {
    use crate::{
        text::stroke_font_test::TEST_FONT_SVG, Angle, Font, Rect, SampleSettings, Text, TextAlign,
        TextLayout, VerticalAlign, Wrap, V2,
    };

    fn font() -> Font {
        Font::from_svg_str(TEST_FONT_SVG).unwrap()
    }

    fn start(layout: &TextLayout, glyph: usize) -> V2 {
        layout.glyphs[glyph].glyph.contours[0]
            .start_point()
            .unwrap()
    }

    fn close(a: V2, b: V2) -> bool {
        a.dist(b) < 1e-4
    }

    #[test]
    fn centered_two_lines() {
        let layout = Text::new("AV\nA", &font())
            .size(1.0)
            .align(TextAlign::Center)
            .layout()
            .unwrap();
        assert_eq!(layout.glyphs.len(), 3);
        assert_eq!(layout.lines.len(), 2);
        assert!((layout.lines[0].width - 1.12).abs() < 1e-6);
        assert_eq!(layout.glyphs[2].line, 1);
        assert_eq!(layout.lines[1].glyphs, 2..3);

        // A starts at x 50 font units, line 0 is centered on the anchor
        assert!(close(start(&layout, 0), V2::new(-0.56 + 0.05, 0.0)));
        assert!(close(start(&layout, 2), V2::new(-0.3 + 0.05, -1.25)));
        assert!(close(layout.lines[1].origin, V2::new(-0.3, -1.25)));
    }

    #[test]
    fn wraps_at_max_width() {
        let layout = Text::new("AA AA AA", &font())
            .max_width(2.8)
            .layout()
            .unwrap();
        assert_eq!(layout.lines.len(), 2);
        assert!((layout.lines[0].width - 2.7).abs() < 1e-6);
        assert!((layout.lines[1].width - 1.2).abs() < 1e-6);
        assert_eq!(layout.glyphs.iter().filter(|g| g.line == 1).count(), 2);
    }

    #[test]
    fn hyphen_break_keeps_the_hyphen_on_the_line() {
        let layout = Text::new("AA-AA AA", &font())
            .max_width(2.0)
            .layout()
            .unwrap();
        assert_eq!(layout.lines.len(), 3);
        assert_eq!(layout.lines[0].glyphs.len(), 3);
        assert!((layout.lines[0].width - 1.7).abs() < 1e-6);
        assert!((layout.lines[1].width - 1.2).abs() < 1e-6);
    }

    #[test]
    fn anywhere_breaks_mid_word_and_drops_leading_spaces() {
        let layout = Text::new("AA AA", &font())
            .max_width(1.3)
            .wrap(Wrap::Anywhere)
            .layout()
            .unwrap();
        assert_eq!(layout.lines.len(), 2);
        assert_eq!(layout.glyphs.len(), 4);
        assert!(layout
            .lines
            .iter()
            .all(|line| (line.width - 1.2).abs() < 1e-6));
        assert!(close(start(&layout, 2), V2::new(0.05, -1.25)));
    }

    #[test]
    fn justify_stretches_all_but_the_last_line() {
        let layout = Text::new("AA AA AA", &font())
            .max_width(2.8)
            .align(TextAlign::Justify)
            .layout()
            .unwrap();
        assert!((layout.lines[0].width - 2.8).abs() < 1e-6);
        assert!((layout.lines[1].width - 1.2).abs() < 1e-6);
        assert!(close(start(&layout, 2), V2::new(1.65, 0.0)));
        assert!(close(start(&layout, 4), V2::new(0.05, -1.25)));
    }

    #[test]
    fn letter_spacing_and_anchor() {
        let layout = Text::new("AA", &font())
            .letter_spacing(0.1)
            .at(V2::new(2.0, 3.0))
            .vertical_align(VerticalAlign::Top)
            .layout()
            .unwrap();
        assert!((layout.lines[0].width - 1.3).abs() < 1e-6);
        assert!(close(
            start(&layout, 1),
            V2::new(2.0 + 0.7 + 0.05, 3.0 - 0.8)
        ));
        let bounds = layout.bounds.unwrap();
        assert!((bounds.tr().y - (3.0 - 0.8 + 0.7)).abs() < 1e-6);
    }

    #[test]
    fn fit_width_includes_letter_spacing() {
        let layout = Text::new("AA\nAAA AA", &font())
            .letter_spacing(0.1)
            .fit_width(5.0)
            .layout()
            .unwrap();
        let size = (5.0 - 0.5) / 3.3;
        assert!((layout.lines[1].width - 5.0).abs() < 1e-5);
        assert!((layout.lines[0].width - (1.2 * size + 0.1)).abs() < 1e-5);
        assert!((layout.glyphs[0].glyph.advance - 0.6 * size).abs() < 1e-5);
    }

    #[test]
    fn fit_inside_fills_the_rect_by_metrics() {
        let rect = Rect::new(V2::zero(), V2::new(3.0, 2.0));
        let layout = Text::new("AA AA AA", &font())
            .fit_inside(rect)
            .vertical_align(VerticalAlign::Top)
            .layout()
            .unwrap();
        // two lines: height (1.0 + 1.25) * size == 2.0 limits before the width does
        let size = layout.glyphs[0].glyph.advance / 0.6;
        assert_eq!(layout.lines.len(), 2);
        assert!((size - 2.0 / 2.25).abs() < 1e-4);
        assert!(layout.lines.iter().all(|line| line.width <= 3.0));
        assert!((layout.lines[0].origin.y - (2.0 - 0.8 * size)).abs() < 1e-4);
        assert!((layout.lines[1].origin.y - 0.2 * size).abs() < 1e-4);

        let bigger = Text::new("AA AA AA", &font())
            .size(size * 1.05)
            .in_rect(rect)
            .vertical_align(VerticalAlign::Top)
            .layout()
            .unwrap();
        assert_eq!(bigger.lines.len(), 2);
        assert!(bigger.lines[1].origin.y - 0.2 * size * 1.05 < 0.0);
    }

    #[test]
    fn in_rect_anchors_to_the_aligned_edges() {
        let rect = Rect::new(V2::new(1.0, 1.0), V2::new(5.0, 4.0));
        let layout = Text::new("AA", &font())
            .in_rect(rect)
            .align(TextAlign::Right)
            .vertical_align(VerticalAlign::Top)
            .layout()
            .unwrap();
        assert!(close(layout.lines[0].origin, V2::new(5.0 - 1.2, 4.0 - 0.8)));
        assert!(close(start(&layout, 0), V2::new(3.85, 3.2)));

        let baseline = Text::new("AA\nAA", &font()).in_rect(rect).layout().unwrap();
        assert!(close(baseline.lines[0].origin, V2::new(1.0, 2.25)));
        assert!(close(baseline.lines[1].origin, V2::new(1.0, 1.0)));
    }

    #[test]
    fn rotation_turns_the_block_around_the_anchor() {
        let layout = Text::new("AA", &font())
            .at(V2::new(2.0, 3.0))
            .rotation(Angle::quarter_rotation())
            .layout()
            .unwrap();
        assert!(close(layout.lines[0].origin, V2::new(2.0, 3.0)));
        assert!(close(start(&layout, 0), V2::new(2.0, 3.05)));
        assert!(close(start(&layout, 1), V2::new(2.0, 3.65)));
    }

    #[test]
    fn layer_has_one_path_per_contour() {
        let layer = Text::new("AV\nA", &font())
            .to_layer(SampleSettings::default())
            .unwrap();
        assert_eq!(layer.shapes.len(), 5);
        assert!(layer.shapes.iter().all(|shape| {
            matches!(shape, crate::Shape::Path(path) if path.get_points_ref().len() >= 2)
        }));
        let by_line = Text::new("AV\nA", &font())
            .layout()
            .unwrap()
            .to_layer_by_line(SampleSettings::default());
        assert_eq!(by_line.sublayers.len(), 2);
        assert_eq!(by_line.sublayers[1].shapes.len(), 2);
    }

    #[test]
    fn cap_height_sets_the_size() {
        let text = Text::new("A", &font()).cap_height(1.4);
        let bounds = text.bounding_box().unwrap().unwrap();
        assert!((bounds.height() - 1.4).abs() < 1e-6);
    }
}
