#[cfg(test)]
mod test_text {
    use crate::{
        text::stroke_font_test::TEST_FONT_SVG, Font, SampleSettings, Text, TextAlign,
        VerticalAlign, V2,
    };

    fn font() -> Font {
        Font::from_svg_str(TEST_FONT_SVG).unwrap()
    }

    #[test]
    fn centered_two_lines() {
        let layout = Text::new("AV\nA", &font())
            .size(1.0)
            .align(TextAlign::Center)
            .layout()
            .unwrap();
        assert_eq!(layout.glyphs.len(), 3);
        assert_eq!(layout.line_widths.len(), 2);
        assert!((layout.line_widths[0] - 1.12).abs() < 1e-6);
        assert_eq!(layout.glyphs[2].line, 1);

        // A starts at x 50 font units, line 0 is centered on the anchor
        let first = layout.glyphs[0].glyph.contours[0].start_point().unwrap();
        assert!((first.x - (-0.56 + 0.05)).abs() < 1e-6);
        assert!(first.y.abs() < 1e-6);
        let second_line = layout.glyphs[2].glyph.contours[0].start_point().unwrap();
        assert!((second_line.y + 1.25).abs() < 1e-6);
    }

    #[test]
    fn wraps_at_max_width() {
        let layout = Text::new("AA AA AA", &font())
            .max_width(2.8)
            .layout()
            .unwrap();
        assert_eq!(layout.line_widths.len(), 2);
        assert!((layout.line_widths[0] - 2.7).abs() < 1e-6);
        assert!((layout.line_widths[1] - 1.2).abs() < 1e-6);
        assert_eq!(layout.glyphs.iter().filter(|g| g.line == 1).count(), 2);
    }

    #[test]
    fn letter_spacing_and_anchor() {
        let layout = Text::new("AA", &font())
            .letter_spacing(0.1)
            .at(V2::new(2.0, 3.0))
            .vertical_align(VerticalAlign::Top)
            .layout()
            .unwrap();
        assert!((layout.line_widths[0] - 1.3).abs() < 1e-6);
        let second = layout.glyphs[1].glyph.contours[0].start_point().unwrap();
        assert!((second.x - (2.0 + 0.7 + 0.05)).abs() < 1e-6);
        assert!((second.y - (3.0 - 0.8)).abs() < 1e-6);
        let bounds = layout.bounds.unwrap();
        assert!((bounds.tr().y - (3.0 - 0.8 + 0.7)).abs() < 1e-6);
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
    }

    #[test]
    fn cap_height_sets_the_size() {
        let text = Text::new("A", &font()).cap_height(1.4);
        let bounds = text.bounding_box().unwrap().unwrap();
        assert!((bounds.height() - 1.4).abs() < 1e-6);
    }
}
