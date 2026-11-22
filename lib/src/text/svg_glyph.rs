use std::io::Read;

use anyhow::{bail, Context, Result};
use svg::{node::element::tag::Type as SvgTagType, parser::Event};

use crate::{
    composition::{svg_elements::element_contours, svg_transform::parse_svg_transform_attr},
    Spline, Transform, TransformMatrix,
};

/// Contours of one OpenType `SVG ` table document in font units, y-up.
///
/// Handles `path`, `line`, `polyline` and `polygon` with `transform` chains; the document is y-down
/// per the spec and is flipped here. Gzip-compressed documents are inflated.
pub(crate) fn parse_svg_glyph(document: &[u8]) -> Result<Vec<Spline>> {
    let inflated;
    let bytes = if document.starts_with(&[0x1f, 0x8b]) {
        let mut buffer = Vec::new();
        flate2::read::GzDecoder::new(document)
            .read_to_end(&mut buffer)
            .context("inflating SVG glyph")?;
        inflated = buffer;
        &inflated
    } else {
        document
    };
    let text = std::str::from_utf8(bytes).context("SVG glyph is not UTF-8")?;

    let flip = TransformMatrix::mirror_y();
    let mut transform_stack = vec![TransformMatrix::identity()];
    let mut contours: Vec<Spline> = Vec::new();
    for event in svg::read(text)? {
        let Event::Tag(tag, tag_type, attributes) = event else {
            if let Event::Error(error) = event {
                bail!("invalid SVG glyph: {error}");
            }
            continue;
        };
        if matches!(tag_type, SvgTagType::End) {
            if transform_stack.len() > 1 {
                transform_stack.pop();
            }
            continue;
        }
        let parent = *transform_stack
            .last()
            .unwrap_or(&TransformMatrix::identity());
        let transform = parent.mul_matrix(
            &parse_svg_transform_attr(&attributes).unwrap_or(TransformMatrix::identity()),
        );
        if matches!(tag_type, SvgTagType::Start) {
            transform_stack.push(transform);
        }
        let to_y_up = flip.mul_matrix(&transform);
        contours.extend(
            element_contours(tag, &attributes)
                .iter()
                .map(|contour| contour.transform(&to_y_up)),
        );
    }
    Ok(contours)
}
