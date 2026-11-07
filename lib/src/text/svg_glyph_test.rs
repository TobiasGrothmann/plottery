#[cfg(test)]
mod test_svg_glyph {
    use std::io::Write;

    use crate::{text::svg_glyph::parse_svg_glyph, V2};

    const FLIPPED_GROUP: &str = r#"<svg xmlns="http://www.w3.org/2000/svg"><g transform="matrix(1 0 0 -1 0 0)"><path fill="none" d="M40 528H666M140 680V0" /></g></svg>"#;

    #[test]
    fn group_flip_and_spec_flip_cancel_out() {
        let contours = parse_svg_glyph(FLIPPED_GROUP.as_bytes()).unwrap();
        assert_eq!(contours.len(), 2);
        assert!(contours.iter().all(|contour| !contour.is_closed()));
        assert_eq!(contours[0].start_point(), Some(V2::new(40.0, 528.0)));
        assert_eq!(contours[0].end_point(), Some(V2::new(666.0, 528.0)));
        assert_eq!(contours[1].start_point(), Some(V2::new(140.0, 680.0)));
        assert_eq!(contours[1].end_point(), Some(V2::new(140.0, 0.0)));
    }

    #[test]
    fn untransformed_document_is_flipped_to_y_up() {
        let doc = r#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M 126 -375 L 430 -375" /><g transform="translate(5 0)"><polygon points="0,0 10,0 10,-10"/></g></svg>"#;
        let contours = parse_svg_glyph(doc.as_bytes()).unwrap();
        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].start_point(), Some(V2::new(126.0, 375.0)));
        assert!(contours[1].is_closed());
        assert_eq!(contours[1].segments()[1].to(), V2::new(15.0, 10.0));
    }

    #[test]
    fn gzip_compressed_document() {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(FLIPPED_GROUP.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();
        let contours = parse_svg_glyph(&compressed).unwrap();
        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].start_point(), Some(V2::new(40.0, 528.0)));
    }
}
