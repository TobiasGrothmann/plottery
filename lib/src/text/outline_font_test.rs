#[cfg(test)]
mod test_outline_font {
    use crate::Font;

    // Not hermetic: uses macOS' Helvetica and passes trivially elsewhere. It is the only way to
    // exercise skrifa and harfrust without vendoring a binary font.
    #[test]
    fn helvetica_outlines_and_kerning() {
        let path = "/System/Library/Fonts/Helvetica.ttc";
        if !std::path::Path::new(path).exists() {
            eprintln!("skipping: {path} not found");
            return;
        }
        let font = Font::load(path).unwrap();
        assert!(!font.is_stroke());
        assert_eq!(font.family_name(), Some("Helvetica"));

        let h = font.glyph('H', 1.0);
        assert_eq!(h.contours.len(), 1);
        assert!(h.contours[0].is_closed());
        assert!(h.advance > 0.0);
        assert_eq!(font.glyph('O', 1.0).contours.len(), 2);

        let metrics = font.metrics(1.0);
        assert!(metrics.ascender > 0.0 && metrics.descender < 0.0);
        assert!(metrics.cap_height.is_some_and(|cap| cap > 0.5 && cap < 0.9));

        let shaped = font.shape("AV", 1.0).unwrap();
        assert_eq!(shaped.len(), 2);
        assert!(shaped[0].advance < font.glyph('A', 1.0).advance);

        let space = font.glyph(' ', 1.0);
        assert!(space.contours.is_empty() && space.advance > 0.0);

        assert!(Font::system("Helvetica").is_ok());
        assert!(Font::system("No Such Family 123").is_err());
    }
}
