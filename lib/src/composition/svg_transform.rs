use svg::node::Attributes;

use crate::{TransformMatrix, V2};

pub(crate) fn iter_svg_f32_tokens(input: &str) -> impl Iterator<Item = f32> + '_ {
    input
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .filter_map(|token| token.parse::<f32>().ok())
}

pub(crate) fn parse_svg_transform_attr(attributes: &Attributes) -> Option<TransformMatrix> {
    parse_svg_transform_chain(&attributes.get("transform")?.to_string())
}

/// A chain like `translate(1 2) scale(3)`; rotate and skew are not supported yet.
pub(crate) fn parse_svg_transform_chain(transform_attr: &str) -> Option<TransformMatrix> {
    let mut combined = TransformMatrix::identity();
    let mut parsed_any = false;
    let mut rest = transform_attr.trim();

    while !rest.is_empty() {
        let Some(open_idx) = rest.find('(') else {
            break;
        };
        let Some(close_idx) = rest.find(')') else {
            break;
        };
        if close_idx <= open_idx {
            break;
        }

        let name = rest[..open_idx].trim();
        let args: Vec<f32> = iter_svg_f32_tokens(&rest[(open_idx + 1)..close_idx]).collect();

        let next_transform = match (name, args.as_slice()) {
            ("matrix", &[a, b, c, d, e, f]) => Some(TransformMatrix {
                tl: a,
                bl: b,
                tr: c,
                br: d,
                u: e,
                v: f,
            }),
            ("translate", &[tx]) => Some(TransformMatrix::translate(V2::new(tx, 0.0))),
            ("translate", &[tx, ty, ..]) => Some(TransformMatrix::translate(V2::new(tx, ty))),
            ("scale", &[s]) => Some(TransformMatrix::scale_2d(V2::xy(s))),
            ("scale", &[sx, sy, ..]) => Some(TransformMatrix::scale_2d(V2::new(sx, sy))),
            _ => {
                // TODO: Support rotate/skew transforms and malformed transform expressions.
                None
            }
        };

        if let Some(next_transform) = next_transform {
            combined = combined.mul_matrix(&next_transform);
            parsed_any = true;
        }

        rest = rest[(close_idx + 1)..].trim_start();
    }

    parsed_any.then_some(combined)
}
