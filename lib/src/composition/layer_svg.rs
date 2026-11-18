use anyhow::{Ok, Result};
use std::path::PathBuf;
use svg::{
    node::{
        element::{path::Data, tag::Type as SvgTagType, Group, Style},
        Attributes,
    },
    parser::Event,
    Document, Node,
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

/// SVG exports use millimetres as user units.
const SVG_UNITS_PER_CM: f32 = 10.0;

impl Layer {
    /// Return the layer as a real-size [`svg::Document`] in mm, y-axis flipped, sublayers as `<g>`, strokes multiplied like ink.
    pub fn to_svg(&self) -> Document {
        // The view box follows the content so shapes below or left of the origin stay visible.
        match self.bounding_box() {
            Some(bounding_box) => self.to_svg_in(bounding_box),
            None => Document::new(),
        }
    }
    /// Like [`Layer::to_svg`], but the document shows exactly `view` instead of the layer's
    /// bounding box. Layers of different extent then line up, e.g. as frames of an animation.
    pub fn to_svg_in(&self, view: Rect) -> Document {
        let height = view.size().only_y();
        let prepared = self.map_recursive(|shape| shape.mirror_y().translate(height));

        let flipped_bl = V2::new(view.bl().x, height.y - view.tr().y);
        let origin: V2 = flipped_bl * SVG_UNITS_PER_CM;
        let size: V2 = view.size() * SVG_UNITS_PER_CM;

        Document::new()
            .set("viewBox", (origin.x, origin.y, size.x, size.y))
            .set("width", format!("{}mm", size.x))
            .set("height", format!("{}mm", size.y))
            // Round like a real pen tip; the default miter join spikes at sharp corners.
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round")
            .add(Style::new("path,circle,rect{mix-blend-mode:multiply}"))
            .add(prepared.get_svg_group(&LayerPropsInheritable::default()))
    }
    fn get_svg_group(&self, parent_props: &LayerPropsInheritable) -> Group {
        let props_inheritable = parent_props.overwrite_with(&self.props_inheritable);

        let mut group = Group::new();
        if let Some(name) = &self.props.name {
            group = group.set("id", name.as_str());
        }

        for shape_svg in self.get_shapes_as_svg_nodes(&props_inheritable) {
            group = group.add(shape_svg);
        }
        for sublayer_svg in self
            .sublayers
            .iter()
            .map(|sublayer| sublayer.get_svg_group(&props_inheritable))
        {
            group = group.add(sublayer_svg);
        }
        group
    }
    fn get_shapes_as_svg_nodes(&self, props: &LayerPropsInheritable) -> Vec<Box<dyn Node>> {
        let fill = "none";
        let stroke = props.color.unwrap().hex();
        let stroke_width = props.pen_width_cm.unwrap() * SVG_UNITS_PER_CM;

        let mut nodes: Vec<Box<dyn Node>> = Vec::with_capacity(self.shapes.len());
        for shape in &self.shapes {
            match shape {
                Shape::Circle(c) => nodes.push(Box::new(
                    svg::node::element::Circle::new()
                        .set("cx", c.center.x * SVG_UNITS_PER_CM)
                        .set("cy", c.center.y * SVG_UNITS_PER_CM)
                        .set("r", c.radius * SVG_UNITS_PER_CM)
                        .set("fill", fill)
                        .set("stroke", stroke.clone())
                        .set("stroke-width", stroke_width),
                )),
                Shape::Rect(r) => {
                    let bl = r.bl();
                    let size = r.size();
                    nodes.push(Box::new(
                        svg::node::element::Rectangle::new()
                            .set("x", bl.x * SVG_UNITS_PER_CM)
                            .set("y", bl.y * SVG_UNITS_PER_CM)
                            .set("width", size.x * SVG_UNITS_PER_CM)
                            .set("height", size.y * SVG_UNITS_PER_CM)
                            .set("fill", fill)
                            .set("stroke", stroke.clone())
                            .set("stroke-width", stroke_width),
                    ));
                }
                Shape::Path(p) => {
                    let points = p.get_points_ref();
                    if points.len() <= 1 {
                        continue;
                    }
                    let mut data = Data::new();
                    data = data.move_to((points[0] * SVG_UNITS_PER_CM).as_tuple());
                    data = points.iter().skip(1).fold(data, |data, point| {
                        data.line_to((point * SVG_UNITS_PER_CM).as_tuple())
                    });

                    if points.first().unwrap() == points.last().unwrap() {
                        data = data.close();
                    }
                    nodes.push(Box::new(
                        svg::node::element::Path::new()
                            .set("d", data)
                            .set("fill", fill)
                            .set("stroke", stroke.clone())
                            .set("stroke-width", stroke_width),
                    ));
                }
            }
        }
        nodes
    }
    /// Save a converted version of this layer to an .svg file. see [`Layer::to_svg`].
    pub fn write_svg(&self, path: PathBuf) -> Result<()> {
        let document = self.to_svg();
        svg::save(path.to_str().unwrap(), &document)?;
        Ok(())
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

                                if path.get_points_ref().len() >= 2 {
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
            layer = layer.map_recursive(|shape| shape.mirror_y().translate(bounds.tr().only_y()));
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
            path.get_points_ref()
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
        let points = path.get_points_ref();
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
