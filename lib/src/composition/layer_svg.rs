use anyhow::{bail, Context, Result};
use std::{fmt::Write, path::PathBuf};
use svg::{
    node::{element::tag::Type as SvgTagType, Attributes},
    parser::Event,
};

use crate::{
    geometry::TransformMatrix,
    traits::{Mirror, Translate},
    BoundingBox, Path, Rect, SampleSettings, Shape, V2,
};

use super::{
    svg_elements::{attr, element_contours},
    svg_path_data::svg_numbers,
    svg_transform::parse_svg_transform_attr,
    Layer, LayerPropsInheritable,
};

/// SVG exports use millimeters as user units.
const SVG_UNITS_PER_CM: f64 = 10.0;

fn mm(cm: f32) -> f64 {
    cm as f64 * SVG_UNITS_PER_CM
}

/// Appends `mm` rounded to a thousandth of a millimeter, without trailing zeros.
pub(super) fn push_mm(out: &mut String, mm: f64) {
    let thousandths = (mm * 1000.0).round() as i64;
    let (whole, fraction) = (thousandths.abs() / 1000, thousandths.abs() % 1000);
    if thousandths < 0 {
        out.push('-');
    }
    let _ = write!(out, "{whole}");
    if fraction != 0 {
        let _ = write!(out, ".{fraction:03}");
        out.truncate(out.trim_end_matches('0').len());
    }
}

/// Appends ` name="value"` with `value` in mm.
fn push_mm_attr(out: &mut String, name: &str, mm: f64) {
    let _ = write!(out, " {name}=\"");
    push_mm(out, mm);
    out.push('"');
}

/// Appends `text` with the characters escaped that end an attribute value or start markup.
fn push_escaped(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
}

/// Writes shapes in SVG coordinates: mm, the y-axis flipped inside a view of `view_height` cm.
struct SvgWriter {
    out: String,
    view_height: f32,
}

impl SvgWriter {
    fn y(&self, y: f32) -> f64 {
        (self.view_height as f64 - y as f64) * SVG_UNITS_PER_CM
    }
    fn push_point(&mut self, point: &V2) {
        let y = self.y(point.y);
        push_mm(&mut self.out, mm(point.x));
        self.out.push(',');
        push_mm(&mut self.out, y);
    }

    /// Stroke color and width are written on the group, and only where they differ from `parent`.
    fn push_layer(&mut self, layer: &Layer, parent: Option<&LayerPropsInheritable>) {
        let inherited = parent.cloned().unwrap_or_default();
        let props = inherited.overwrite_with(&layer.props_inheritable);
        let (color, pen_width) = (props.color.unwrap(), props.pen_width.unwrap());

        self.out.push_str("<g");
        if let Some(name) = &layer.props.name {
            self.out.push_str(" id=\"");
            push_escaped(&mut self.out, name);
            self.out.push('"');
        }
        if parent.is_none() || color != inherited.color.unwrap() {
            let _ = write!(self.out, " stroke=\"{}\"", color.hex());
        }
        if parent.is_none() || pen_width != inherited.pen_width.unwrap() {
            push_mm_attr(&mut self.out, "stroke-width", mm(pen_width));
        }
        self.out.push_str(">\n");

        for shape in &layer.shapes {
            self.push_shape(shape);
        }
        for sublayer in &layer.sublayers {
            self.push_layer(sublayer, Some(&props));
        }
        self.out.push_str("</g>\n");
    }

    fn push_shape(&mut self, shape: &Shape) {
        match shape {
            Shape::Circle(c) => {
                let cy = self.y(c.center.y);
                self.out.push_str("<circle");
                push_mm_attr(&mut self.out, "cx", mm(c.center.x));
                push_mm_attr(&mut self.out, "cy", cy);
                push_mm_attr(&mut self.out, "r", mm(c.radius));
                self.out.push_str("/>\n");
            }
            Shape::Rect(r) => {
                // The top left corner, as the y-axis points down.
                let y = self.y(r.tr().y);
                self.out.push_str("<rect");
                push_mm_attr(&mut self.out, "x", mm(r.bl().x));
                push_mm_attr(&mut self.out, "y", y);
                push_mm_attr(&mut self.out, "width", mm(r.width()));
                push_mm_attr(&mut self.out, "height", mm(r.height()));
                self.out.push_str("/>\n");
            }
            Shape::Path(p) => {
                let points = p.as_slice();
                let [first, rest @ ..] = points else {
                    return;
                };
                if rest.is_empty() {
                    return;
                }
                self.out.push_str("<path d=\"M");
                self.push_point(first);
                // One `L`, the points after it are lines too.
                self.out.push_str(" L");
                for (i, point) in rest.iter().enumerate() {
                    if i > 0 {
                        self.out.push(' ');
                    }
                    self.push_point(point);
                }
                if points.first() == points.last() {
                    self.out.push_str(" z");
                }
                self.out.push_str("\"/>\n");
            }
        }
    }
}

impl Layer {
    /// Return the layer as a real-size SVG document in mm, y-axis flipped, sublayers as `<g>`, strokes multiplied like ink.
    pub fn to_svg(&self) -> String {
        // The view box follows the content so shapes below or left of the origin stay visible.
        match self.bounding_box() {
            Some(bounding_box) => self.to_svg_in(bounding_box),
            None => "<svg xmlns=\"http://www.w3.org/2000/svg\"/>\n".to_string(),
        }
    }
    /// Like [`Layer::to_svg`], but the document shows exactly `view` instead of the layer's
    /// bounding box. Layers of different extent then line up, e.g. as frames of an animation.
    pub fn to_svg_in(&self, view: Rect) -> String {
        let mut writer = SvgWriter {
            out: String::new(),
            view_height: view.height(),
        };
        // The view's top left corner, where the flipped y-axis starts.
        let (x, y) = (mm(view.bl().x), writer.y(view.tr().y));
        let (width, height) = (mm(view.width()), mm(view.height()));

        let out = &mut writer.out;
        out.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"");
        for (i, value) in [x, y, width, height].into_iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            push_mm(out, value);
        }
        out.push_str("\" width=\"");
        push_mm(out, width);
        out.push_str("mm\" height=\"");
        push_mm(out, height);
        // Round like a real pen tip; the default miter join spikes at sharp corners.
        out.push_str("mm\" fill=\"none\" stroke-linejoin=\"round\" stroke-linecap=\"round\">\n");
        out.push_str("<style>path,circle,rect{mix-blend-mode:multiply}</style>\n");

        writer.push_layer(self, None);
        writer.out.push_str("</svg>\n");
        writer.out
    }
    /// Save a converted version of this layer to an .svg file. see [`Layer::to_svg`].
    pub fn write_svg(&self, path: PathBuf) -> Result<()> {
        std::fs::write(&path, self.to_svg())
            .with_context(|| format!("Failed to write SVG '{}'", path.display()))
    }

    /// Creates a new `Layer` from an .svg file.
    ///
    /// Imports `path` (all commands, arcs as straight lines), `line`, `polyline` and `polygon`
    /// with their `transform`s, scaled to cm by the root's `viewBox` and size. Filled paths are closed.
    pub fn new_from_svg(path: &PathBuf) -> Result<Layer> {
        let mut svg_content = String::new();
        let mut layer = Layer::new();
        let mut transform_stack: Vec<TransformMatrix> = vec![TransformMatrix::identity()];
        let mut svg_units_to_cm_scale: f32 = 1.0;

        for event in svg::open(path, &mut svg_content)? {
            let Event::Tag(tag, tag_type, attributes) = event else {
                if let Event::Error(error) = event {
                    bail!("invalid SVG '{}': {error}", path.display());
                }
                continue;
            };
            if matches!(tag_type, SvgTagType::End) {
                if transform_stack.len() > 1 {
                    transform_stack.pop();
                }
                continue;
            }
            if tag == "svg" {
                if let Some(scale) = svg_root_units_to_cm_scale(&attributes) {
                    svg_units_to_cm_scale = scale;
                }
            }

            let parent_transform = *transform_stack
                .last()
                .unwrap_or(&TransformMatrix::identity());
            let transform = parent_transform.mul_matrix(
                &parse_svg_transform_attr(&attributes).unwrap_or(TransformMatrix::identity()),
            );
            if matches!(tag_type, SvgTagType::Start) {
                transform_stack.push(transform);
            }

            let close_filled = tag == "path" && svg_path_has_fill(&attributes);
            let sample_settings = SampleSettings::new(
                SampleSettings::default().points_per_unit * svg_units_to_cm_scale,
            );
            for contour in element_contours(tag, &attributes) {
                let mut path = contour.to_path(sample_settings);
                if close_filled {
                    close_path_if_needed(&mut path);
                }
                if path.len() >= 2 {
                    layer.push(
                        path.iter()
                            .map(|&point| transform.mul_vector(point) * svg_units_to_cm_scale)
                            .collect::<Path>(),
                    );
                }
            }
        }

        // Convert SVG y-down coordinates back to Plottery y-up coordinates.
        if let Some(bounds) = layer.bounding_box() {
            layer =
                layer.map_recursive_par(|shape| shape.mirror_y().translate(bounds.tr().only_y()));
        }

        Ok(layer)
    }
}

/// cm per SVG user unit from the root's `viewBox` and `width`/`height`, averaged over both axes.
fn svg_root_units_to_cm_scale(attributes: &Attributes) -> Option<f32> {
    let mut view_box = svg_numbers(attr(attributes, "viewBox")?).skip(2);
    let (view_box_w, view_box_h) = (view_box.next()?, view_box.next()?);
    let scale = |length: Option<&str>, view_box_size: f32| {
        (view_box_size.abs() > f32::EPSILON)
            .then(|| svg_length_to_cm(length?).map(|cm| cm / view_box_size))
            .flatten()
    };
    match (
        scale(attr(attributes, "width"), view_box_w),
        scale(attr(attributes, "height"), view_box_h),
    ) {
        (Some(sx), Some(sy)) => Some((sx + sy) * 0.5),
        (sx, sy) => sx.or(sy),
    }
}

/// A length like `21cm` or `5.5in` in cm; plain numbers and unknown units are `None`.
fn svg_length_to_cm(length: &str) -> Option<f32> {
    let trimmed = length.trim();
    let split_at = trimmed
        .find(|ch: char| !(ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+' | 'e' | 'E')))
        .unwrap_or(trimmed.len());
    let value = trimmed[..split_at].trim().parse::<f32>().ok()?;
    let cm_per_unit = match trimmed[split_at..].trim().to_ascii_lowercase().as_str() {
        "cm" => 1.0,
        "mm" => 0.1,
        "in" => 2.54,
        "px" => 2.54 / 96.0,
        "pt" => 2.54 / 72.0,
        "pc" => 2.54 / 6.0,
        "q" => 0.025, // quarter-millimeter
        _ => return None,
    };
    Some(value * cm_per_unit)
}

fn close_path_if_needed(path: &mut Path) {
    if let [first, .., last] = path.as_slice() {
        if path.len() >= 3 && first != last {
            let first = *first;
            path.push(first);
        }
    }
}

/// True if a `fill` attribute or style declaration sets a fill other than `none`.
fn svg_path_has_fill(attributes: &Attributes) -> bool {
    let is_fill = |value: &str| {
        let value = value.trim();
        !value.is_empty() && !value.eq_ignore_ascii_case("none")
    };
    attr(attributes, "fill").is_some_and(is_fill)
        || attr(attributes, "style").is_some_and(|style| {
            style
                .to_ascii_lowercase()
                .split(';')
                .any(|part| part.trim().strip_prefix("fill:").is_some_and(is_fill))
        })
}
