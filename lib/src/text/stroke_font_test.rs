#[cfg(test)]
pub(crate) const TEST_FONT_SVG: &str = r#"<?xml version="1.0" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg">
<defs>
<font id="TestFont" horiz-adv-x="500">
<font-face font-family="Test Stroke" units-per-em="1000" ascent="800" descent="-200" cap-height="700" x-height="500"/>
<missing-glyph horiz-adv-x="500"/>
<glyph unicode=" " glyph-name="space" horiz-adv-x="300"/>
<glyph unicode="A" glyph-name="A" horiz-adv-x="600" d="M 50 0 L 300 700 L 550 0 M 150 250 L 450 250"/>
<glyph unicode="V" glyph-name="V" horiz-adv-x="600" d="M 50 700 L 300 0 L 550 700"/>
<glyph unicode="&quot;" glyph-name="quotedbl" horiz-adv-x="300" d="M 100 700 L 100 500 M 200 700 L 200 500"/>
<hkern u1="A" u2="V" k="80"/>
</font>
</defs>
</svg>"#;

#[cfg(test)]
mod test_stroke_font {
    use super::TEST_FONT_SVG;
    use crate::{text::stroke_font::decode_xml_entities, Font, GlyphId, V2};

    fn font() -> Font {
        Font::from_svg_str(TEST_FONT_SVG).unwrap()
    }

    #[test]
    fn glyph_contours_and_advance_are_scaled_to_em() {
        let glyph = font().glyph('A', 1.0);
        assert_eq!(glyph.contours.len(), 2);
        assert!((glyph.advance - 0.6).abs() < 1e-6);
        assert_eq!(glyph.contours[0].start_point(), Some(V2::new(0.05, 0.0)));
        assert_eq!(glyph.contours[0].len(), 2);
        assert!(!glyph.contours[0].is_closed());
    }

    #[test]
    fn unicode_attribute_is_entity_decoded() {
        let font = font();
        assert!(font.has_glyph('"'));
        assert_eq!(font.glyph('"', 1.0).contours.len(), 2);
    }

    #[test]
    fn kerning_shortens_the_left_advance() {
        let shaped = font().shape("AV", 1.0).unwrap();
        assert_eq!(shaped.len(), 2);
        assert!((shaped[0].advance - 0.52).abs() < 1e-6);
        assert!((shaped[1].advance - 0.6).abs() < 1e-6);
    }

    #[test]
    fn unknown_char_is_the_missing_glyph() {
        let font = font();
        assert!(!font.has_glyph('Z'));
        let glyph = font.glyph('Z', 2.0);
        assert_eq!(glyph.id, GlyphId(0));
        assert!((glyph.advance - 1.0).abs() < 1e-6);
        assert!(glyph.contours.is_empty());
    }

    #[test]
    fn metrics_and_family() {
        let font = font();
        assert!(font.is_stroke());
        assert_eq!(font.family_name(), Some("Test Stroke"));
        let metrics = font.metrics(1.0);
        assert!((metrics.cap_height.unwrap() - 0.7).abs() < 1e-6);
        assert!((metrics.x_height.unwrap() - 0.5).abs() < 1e-6);
        assert!((metrics.ascender - 0.8).abs() < 1e-6);
        assert!((metrics.descender + 0.2).abs() < 1e-6);
    }

    #[test]
    fn missing_font_tag_is_an_error() {
        assert!(Font::from_svg_str("<svg xmlns='http://www.w3.org/2000/svg'></svg>").is_err());
    }

    #[test]
    fn entities() {
        assert_eq!(
            decode_xml_entities("&amp;&#65;&#x42;&quot;&unknown;&"),
            "&AB\"&unknown;&"
        );
    }
}
