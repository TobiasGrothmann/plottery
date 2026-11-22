use svg::node::Attributes;

use super::{svg_path_data::parse_svg_path_data, svg_transform::iter_svg_f32_tokens};
use crate::{shapes::spline::sink::SplineSink, Spline, V2};

pub(crate) fn attr<'a>(attributes: &'a Attributes, key: &str) -> Option<&'a str> {
    attributes.get(key).map(|value| &**value)
}

pub(crate) fn attr_f32(attributes: &Attributes, key: &str) -> Option<f32> {
    attr(attributes, key)?.trim().parse().ok()
}

/// Contours of a `path`, `line`, `polyline` or `polygon` element in its own units, none for other tags.
pub(crate) fn element_contours(tag: &str, attributes: &Attributes) -> Vec<Spline> {
    let number = |key: &str| attr_f32(attributes, key);
    match tag {
        "path" => attr(attributes, "d").map_or_else(Vec::new, parse_svg_path_data),
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
            let values: Vec<f32> =
                iter_svg_f32_tokens(attr(attributes, "points").unwrap_or_default()).collect();
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
