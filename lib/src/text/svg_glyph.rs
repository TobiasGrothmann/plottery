use std::io::Read;

use anyhow::{bail, Context, Result};
use svg::{
    node::{element::tag::Type as SvgTagType, Attributes},
    parser::Event,
};

use crate::{
    composition::{
        svg_path_data::parse_svg_path_data,
        svg_transform::{iter_svg_f32_tokens, parse_svg_transform_attr},
    },
    shapes::spline::sink::SplineSink,
    Spline, Transform, TransformMatrix, V2,
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

fn element_contours(tag: &str, attributes: &Attributes) -> Vec<Spline> {
    let attr = |key: &str| attributes.get(key).map(|value| value.to_string());
    let number = |key: &str| attr(key).and_then(|value| value.trim().parse::<f32>().ok());
    match tag {
        "path" => attr("d").map_or_else(Vec::new, |data| parse_svg_path_data(&data)),
        "line" => match (number("x1"), number("y1"), number("x2"), number("y2")) {
            (Some(x1), Some(y1), Some(x2), Some(y2)) => {
                let mut sink = SplineSink::new();
                sink.move_to(V2::new(x1, y1));
                sink.line_to(V2::new(x2, y2));
                sink.finish()
            }
            _ => Vec::new(),
        },
        "polyline" | "polygon" => {
            let points = attr("points").unwrap_or_default();
            let values: Vec<f32> = iter_svg_f32_tokens(&points).collect();
            let mut sink = SplineSink::new();
            for (i, pair) in values.chunks_exact(2).enumerate() {
                let point = V2::new(pair[0], pair[1]);
                if i == 0 {
                    sink.move_to(point);
                } else {
                    sink.line_to(point);
                }
            }
            if tag == "polygon" {
                sink.close();
            }
            sink.finish()
        }
        _ => Vec::new(),
    }
}
