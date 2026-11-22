use anyhow::{Context, Ok, Result};
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
    svg_path_data::parse_svg_path_data,
    svg_transform::{iter_svg_f32_tokens, parse_svg_transform_attr},
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
    /// Currently this only imports basic `<path d="...">` commands (`M/m`, `L/l`, `H/h`, `V/v`, `Z/z`).
    pub fn new_from_svg(path: &PathBuf) -> Result<Layer> {
        let mut svg_content = String::new();
        let mut layer = Layer::new();
        let mut transform_stack: Vec<TransformMatrix> = vec![TransformMatrix::identity()];
        let mut svg_units_to_cm_scale: f32 = 1.0;

        for event in svg::open(path, &mut svg_content)? {
            if let Event::Tag(tag, tag_type, attributes) = event {
                if tag == "svg" && !matches!(tag_type, SvgTagType::End) {
                    if let Some(scale) = Self::parse_svg_root_units_to_cm_scale(&attributes) {
                        svg_units_to_cm_scale = scale;
                    }
                }

                let parent_transform = *transform_stack
                    .last()
                    .unwrap_or(&TransformMatrix::identity());
                let local_transform =
                    parse_svg_transform_attr(&attributes).unwrap_or(TransformMatrix::identity());
                let combined_transform = parent_transform.mul_matrix(&local_transform);

                if matches!(tag_type, SvgTagType::Start) {
                    transform_stack.push(combined_transform);
                }

                if !matches!(tag_type, SvgTagType::End) {
                    match tag {
                        "path" => {
                            let Some(data_value) = attributes.get("d") else {
                                // TODO: Handle malformed `<path>` nodes without `d` attribute more explicitly.
                                continue;
                            };

                            let path_data = data_value.to_string();
                            let should_close_paths = Self::svg_path_has_fill(&attributes);
                            let sample_settings = SampleSettings::new(
                                SampleSettings::default().points_per_unit * svg_units_to_cm_scale,
                            );
                            for mut path in parse_svg_path_data(path_data.as_str())
                                .iter()
                                .map(|spline| spline.to_path(sample_settings))
                            {
                                if should_close_paths {
                                    Self::close_path_if_needed(&mut path);
                                }

                                if path.as_slice().len() >= 2 {
                                    layer.push(Self::apply_svg_transform_to_path(
                                        &path,
                                        combined_transform,
                                        svg_units_to_cm_scale,
                                    ));
                                }
                            }
                        }
                        "line" => {
                            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                                Self::parse_svg_attr_f32(&attributes, "x1"),
                                Self::parse_svg_attr_f32(&attributes, "y1"),
                                Self::parse_svg_attr_f32(&attributes, "x2"),
                                Self::parse_svg_attr_f32(&attributes, "y2"),
                            ) {
                                layer.push(Self::apply_svg_transform_to_path(
                                    &Path::new_from(vec![V2::new(x1, y1), V2::new(x2, y2)]),
                                    combined_transform,
                                    svg_units_to_cm_scale,
                                ));
                            } else {
                                // TODO: Handle malformed `<line>` nodes more explicitly.
                            }
                        }
                        "polyline" | "polygon" => {
                            let Some(points_attr) = attributes.get("points") else {
                                // TODO: Handle malformed polyline/polygon nodes without points attribute.
                                continue;
                            };

                            let mut points = Self::parse_svg_points_attr(points_attr);
                            if tag == "polygon"
                                && points.len() >= 2
                                && points.first() != points.last()
                            {
                                points.push(*points.first().unwrap());
                            }

                            if points.len() >= 2 {
                                layer.push(Self::apply_svg_transform_to_path(
                                    &Path::new_from(points),
                                    combined_transform,
                                    svg_units_to_cm_scale,
                                ));
                            }
                        }
                        _ => {
                            // TODO: Support importing non-path SVG elements (`circle`, `rect`, etc.).
                        }
                    }
                }

                if matches!(tag_type, SvgTagType::End) && transform_stack.len() > 1 {
                    transform_stack.pop();
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

    fn parse_svg_attr_f32(attributes: &Attributes, key: &str) -> Option<f32> {
        let value = attributes.get(key)?;
        value.to_string().parse::<f32>().ok()
    }

    fn parse_svg_points_attr(points_attr: &svg::node::Value) -> Vec<V2> {
        let points = points_attr.to_string();
        let mut tokens = iter_svg_f32_tokens(points.as_str());
        let mut parsed_points = Vec::new();

        while let Some(x) = tokens.next() {
            let Some(y) = tokens.next() else {
                break;
            };
            parsed_points.push(V2::new(x, y));
        }

        parsed_points
    }

    fn apply_svg_transform_to_path(
        path: &Path,
        transform: TransformMatrix,
        scale_to_cm: f32,
    ) -> Path {
        Path::new_from(
            path.as_slice()
                .iter()
                .map(|point| transform.mul_vector(*point) * scale_to_cm),
        )
    }

    fn parse_svg_root_units_to_cm_scale(attributes: &Attributes) -> Option<f32> {
        let view_box = attributes.get("viewBox")?.to_string();
        let mut view_box_values = iter_svg_f32_tokens(view_box.as_str());
        let _view_box_x = view_box_values.next()?;
        let _view_box_y = view_box_values.next()?;
        let view_box_w = view_box_values.next()?;
        let view_box_h = view_box_values.next()?;

        let width_cm = attributes
            .get("width")
            .and_then(|w| Self::parse_svg_length_to_cm(w.to_string().as_str()));
        let height_cm = attributes
            .get("height")
            .and_then(|h| Self::parse_svg_length_to_cm(h.to_string().as_str()));

        let scale_x = if view_box_w.abs() > f32::EPSILON {
            width_cm.map(|w| w / view_box_w)
        } else {
            None
        };
        let scale_y = if view_box_h.abs() > f32::EPSILON {
            height_cm.map(|h| h / view_box_h)
        } else {
            None
        };

        match (scale_x, scale_y) {
            (Some(sx), Some(sy)) => Some((sx + sy) * 0.5),
            (Some(sx), None) => Some(sx),
            (None, Some(sy)) => Some(sy),
            (None, None) => None,
        }
    }

    fn parse_svg_length_to_cm(length: &str) -> Option<f32> {
        let trimmed = length.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut split_at = trimmed.len();
        for (idx, ch) in trimmed.char_indices() {
            if !(ch.is_ascii_digit()
                || ch == '.'
                || ch == '-'
                || ch == '+'
                || ch == 'e'
                || ch == 'E')
            {
                split_at = idx;
                break;
            }
        }

        let value_str = trimmed[..split_at].trim();
        let unit_str = trimmed[split_at..].trim().to_ascii_lowercase();
        let value = value_str.parse::<f32>().ok()?;

        match unit_str.as_str() {
            "cm" => Some(value),
            "mm" => Some(value * 0.1),
            "in" => Some(value * 2.54),
            "px" => Some(value * (2.54 / 96.0)),
            "pt" => Some(value * (2.54 / 72.0)),
            "pc" => Some(value * (2.54 / 6.0)),
            "q" => Some(value * 0.025), // quarter-millimeter
            _ => {
                // TODO: Support additional SVG/CSS units or project-level unit defaults.
                None
            }
        }
    }

    fn close_path_if_needed(path: &mut Path) {
        let points = path.as_slice();
        if points.len() >= 3 {
            if let (Some(first), Some(last)) = (points.first(), points.last()) {
                if first != last {
                    path.push(*first);
                }
            }
        }
    }

    fn svg_path_has_fill(attributes: &Attributes) -> bool {
        if let Some(fill_value) = attributes.get("fill") {
            let fill = fill_value.to_string().trim().to_ascii_lowercase();
            if !fill.is_empty() && fill != "none" {
                return true;
            }
        }

        if let Some(style_value) = attributes.get("style") {
            let style = style_value.to_string().to_ascii_lowercase();
            for part in style.split(';') {
                let part = part.trim();
                if let Some(value) = part.strip_prefix("fill:") {
                    let fill = value.trim();
                    if !fill.is_empty() && fill != "none" {
                        return true;
                    }
                }
            }
        }

        false
    }
}
