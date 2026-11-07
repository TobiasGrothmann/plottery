#[cfg(test)]
mod test_bundled {
    use crate::{BoundingBox, BundledFont, Font, SampleSettings, Text};

    #[test]
    fn every_bundled_font_loads_and_lays_out() {
        for &bundled in BundledFont::ALL {
            let font = Font::bundled(bundled);
            assert_eq!(font.family_name(), Some(bundled.family()), "{bundled:?}");
            let layer = Text::new("Ab 1", &font)
                .to_layer(SampleSettings::default())
                .unwrap_or_else(|error| panic!("{bundled:?}: {error:#}"));
            assert!(!layer.shapes.is_empty(), "{bundled:?} drew nothing");
        }
    }

    #[test]
    fn relief_truetype_contours_stay_open_and_kern() {
        let font = Font::bundled(BundledFont::ReliefSingleLine);
        assert!(font.is_single_line());
        let l = font.glyph('l', 1.0);
        assert_eq!(l.contours.len(), 1);
        assert!(!l.contours[0].is_closed());
        let shaped = font.shape("AV", 1.0).unwrap();
        assert!(shaped[0].advance < font.glyph('A', 1.0).advance);
    }

    #[test]
    fn mistral_strokes_come_from_the_svg_table() {
        let font = Font::bundled(BundledFont::MistralSingleLine);
        let l = font.glyph('l', 1.0);
        assert!(!l.contours.is_empty());
        assert!(l.contours.iter().all(|contour| !contour.is_closed()));
        let bounds = font.glyph('H', 1.0).bounding_box().unwrap();
        assert!(bounds.tr().y > 0.3, "glyph is upright: {bounds:?}");
    }
}
