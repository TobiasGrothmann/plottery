use anyhow::{Ok, Result};
use bincode::{deserialize_from, serialize};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{
    collections::BTreeSet, fs::File, io::Write, iter::FromIterator, path::PathBuf, slice::Iter, vec,
};
use svg::{
    node::{
        element::{path::Data, tag::Type as SvgTagType, Group},
        Attributes,
    },
    parser::Event,
    Document, Node,
};

use crate::{
    traits::{Normalize, Scale, Scale2D, Translate},
    Angle, BoundingBox, Circle, Masked, Mirror, Path, Plottable, Rect, Rotate, SampleSettings,
    Shape, V2,
};

use super::{path_end::PathEnd, ColorRgb, Inheritable, LayerProps, LayerPropsInheritable};

/// `Layer` represents a tree of [`Shape`]s by holding a list of [`Shape`]s and other `Layer`s.
///
/// ### Example
/// ```
/// # use plottery_lib::*;
///
/// let mut layer = Layer::new();
/// layer.push(Circle::new_shape(V2::new(0.0, 0.0), 1.0));
/// layer.push(Rect::new_shape(V2::new(0.0, 0.0), V2::new(1.0, 1.0)));
///
/// let mut sublayer = Layer::new();
/// sublayer.push(Circle::new_shape(V2::new(3.0, 3.0), 5.0));
///
/// layer.push_layer(sublayer);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub shapes: Vec<Shape>,
    pub sublayers: Vec<Layer>,
    pub props: LayerProps,
    pub props_inheritable: Inheritable<LayerPropsInheritable>,
}

#[derive(Debug, Clone, Copy)]
enum SvgPathToken {
    Command(char),
    Number(f32),
}

#[derive(Debug, Clone, Copy)]
struct SvgTransform {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl SvgTransform {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: 0.0,
            f: 0.0,
        }
    }

    fn translate(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            e: tx,
            f: ty,
        }
    }

    fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,
            b: 0.0,
            c: 0.0,
            d: sy,
            e: 0.0,
            f: 0.0,
        }
    }

    fn matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    fn multiply(self, rhs: Self) -> Self {
        Self {
            a: self.a * rhs.a + self.c * rhs.b,
            b: self.b * rhs.a + self.d * rhs.b,
            c: self.a * rhs.c + self.c * rhs.d,
            d: self.b * rhs.c + self.d * rhs.d,
            e: self.a * rhs.e + self.c * rhs.f + self.e,
            f: self.b * rhs.e + self.d * rhs.f + self.f,
        }
    }

    fn apply(self, point: V2) -> V2 {
        V2::new(
            self.a * point.x + self.c * point.y + self.e,
            self.b * point.x + self.d * point.y + self.f,
        )
    }
}

impl Layer {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            sublayers: Vec::new(),
            props_inheritable: Inheritable::Inherit,
            props: LayerProps::default(),
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            shapes: Vec::with_capacity(capacity),
            sublayers: Vec::new(),
            props_inheritable: Inheritable::Inherit,
            props: LayerProps::default(),
        }
    }
    /// Creates a new `Layer` from a list of shapes.
    pub fn new_from(shapes: Vec<Shape>) -> Self {
        Self {
            shapes,
            sublayers: Vec::new(),
            props_inheritable: Inheritable::Inherit,
            props: LayerProps::default(),
        }
    }
    /// Creates a new `Layer` from both a list of shapes and a list of sublayers.
    pub fn new_from_shapes_and_layers(shapes: Vec<Shape>, sublayers: Vec<Layer>) -> Self {
        Self {
            shapes,
            sublayers,
            props_inheritable: Inheritable::Inherit,
            props: LayerProps::default(),
        }
    }
    /// Creates a new `Layer` by deserializing binary from a file. see [`Layer::write_file`].
    pub fn new_from_file(path: &PathBuf) -> Result<Layer> {
        let file = File::open(path)?;
        let decoded: Layer = deserialize_from(&file)?;
        Ok(decoded)
    }
    /// Writes the binary representation of the `Layer` to a file. see [`Layer::new_from_file`].
    pub fn write_file(&self, path: &PathBuf) -> Result<()> {
        let encoded: Vec<u8> = serialize(self)?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }
    /// Creates a new `Layer` by deserializing binary from a vector of bytes. see [`Layer::to_binary`].
    pub fn new_from_binary(binary_data: &Vec<u8>) -> Result<Layer> {
        Ok(deserialize_from(binary_data.as_slice())?)
    }
    /// Serializes the `Layer` to a vector of bytes. see [`Layer::new_from_binary`].
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        Ok(serialize(self)?)
    }

    /// set the inheritable properties of the layer
    pub fn set_props_inheritable(&mut self, props: Inheritable<LayerPropsInheritable>) {
        self.props_inheritable = props;
    }
    /// set the inheritable properties of the layer
    pub fn with_props_inheritable(mut self, props: Inheritable<LayerPropsInheritable>) -> Self {
        self.set_props_inheritable(props);
        self
    }
    /// set the non-inheritable properties of the layer
    pub fn set_props(&mut self, props: LayerProps) {
        self.props = props;
    }
    /// set the non-inheritable properties of the layer
    pub fn with_props(mut self, props: LayerProps) -> Self {
        self.set_props(props);
        self
    }

    /// Helper to set the color of the layer. see [`Layer::with_props_inheritable`].
    pub fn with_color(mut self, color: ColorRgb) -> Self {
        self.props_inheritable = self
            .props_inheritable
            .overwrite_with(&Inheritable::Specified(
                LayerPropsInheritable::inherit_all().with_color(color),
            ));
        self
    }
    /// Helper to set the pen width of the layer. see [`Layer::with_props_inheritable`].
    pub fn with_pen_width_cm(mut self, pen_width_cm: f32) -> Self {
        self.props_inheritable = self
            .props_inheritable
            .overwrite_with(&Inheritable::Specified(
                LayerPropsInheritable::inherit_all().with_pen_width_cm(pen_width_cm),
            ));
        self
    }
    /// Helper to set the name of the layer. see [`Layer::with_props`].
    pub fn with_name(mut self, name: &str) -> Self {
        self.props = self.props.with_name(name);
        self
    }

    pub fn push<S: Into<Shape>>(&mut self, shape: S) {
        self.shapes.push(shape.into());
    }
    /// Add a layer as sublayer.
    pub fn push_layer<L: Into<Layer>>(&mut self, sublayer: L) {
        self.sublayers.push(sublayer.into());
    }
    /// Add all the [`Shape`]s of the `sublayer` recursively to `self`.
    pub fn push_layer_flat<L: Into<Layer>>(&mut self, sublayer: L) {
        let layer = sublayer.into();
        self.shapes.reserve(layer.len_recursive());
        Self::append_flattened_shapes_owned(layer, &mut self.shapes);
    }

    fn append_flattened_shapes_owned(layer: Layer, out: &mut Vec<Shape>) {
        let Layer {
            shapes, sublayers, ..
        } = layer;

        out.extend(shapes);
        for sublayer in sublayers {
            Self::append_flattened_shapes_owned(sublayer, out);
        }
    }
    pub fn push_many<I, S>(&mut self, shapes: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<Shape>,
    {
        self.shapes.extend(shapes.into_iter().map(Into::into));
    }

    /// Returns an iterator over the shapes of the layer, excluding sublayers.
    pub fn iter(&self) -> Iter<'_, Shape> {
        self.shapes.iter()
    }
    /// Returns an iterator over the Sublayers of the layer, non recursively.
    pub fn iter_sublayers(&self) -> Iter<'_, Layer> {
        self.sublayers.iter()
    }
    /// Returns an iterator over all the shapes of the layer, recursively also iterating all sublayers.
    pub fn iter_flattened(&self) -> LayerFlattenedIterator<'_> {
        LayerFlattenedIterator::new(self)
    }

    /// Returns the number of shapes in the layer, excluding sublayers.
    pub fn len(&self) -> usize {
        self.shapes.len()
    }
    /// Returns true if the layer has no [`Shape]s and no sublayers.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty() && self.sublayers.is_empty()
    }
    /// Returns the number of shapes in the layer, recursively also counting all sublayers.
    pub fn len_recursive(&self) -> usize {
        self.sublayers
            .iter()
            .fold(self.len(), |acc, sublayer| acc + sublayer.len_recursive())
    }
    /// Returns the number of sublayers in the layer, non recursively.
    pub fn len_sublayers(&self) -> usize {
        self.sublayers.len()
    }

    /// Return the layer converted into a [`svg::Document`].
    /// Some adjustments are made, like flipping the y-axis and optinally scaling the shapes.
    /// Sublayers are grouped with an svg `<g>` tag.
    pub fn to_svg(&self, scale: f32) -> Document {
        let prepared = self.get_prepared_for_svg();

        let bounding_box = prepared.bounding_box();
        if bounding_box.is_none() {
            return Document::new();
        }
        let bounding_box = bounding_box.unwrap();
        let svg_max_coords: V2 = bounding_box.tr() * scale;

        Document::new()
            .set("viewBox", (0, 0, svg_max_coords.x, svg_max_coords.y))
            .set("width", svg_max_coords.x)
            .set("height", svg_max_coords.y)
            .add(prepared.get_svg_group(scale, &LayerPropsInheritable::default()))
    }
    fn get_prepared_for_svg(&self) -> Layer {
        let bounding_box = self.bounding_box();
        if bounding_box.is_none() {
            return Layer::new();
        }
        let bounding_box = bounding_box.unwrap();

        self.map_recursive(|shape| shape.mirror_y().translate(bounding_box.size().only_y()))
    }
    fn get_svg_group(&self, scale: f32, parent_props: &LayerPropsInheritable) -> Group {
        let props_inheritable = parent_props.overwrite_with(&self.props_inheritable);

        let mut group = Group::new();
        if let Some(name) = &self.props.name {
            group = group.set("id", name.as_str());
        }

        for shape_svg in self.get_shapes_as_svg_nodes(scale, &props_inheritable) {
            group = group.add(shape_svg);
        }
        for sublayer_svg in self
            .sublayers
            .iter()
            .map(|sublayer| sublayer.get_svg_group(scale, &props_inheritable))
        {
            group = group.add(sublayer_svg);
        }
        group
    }
    fn get_shapes_as_svg_nodes(
        &self,
        scale: f32,
        props: &LayerPropsInheritable,
    ) -> Vec<Box<dyn Node>> {
        let fill = "none";
        let stroke = props.color.unwrap().hex();
        let stroke_width = props.pen_width_cm.unwrap() * scale;

        let mut nodes: Vec<Box<dyn Node>> = Vec::with_capacity(self.shapes.len());
        for shape in self.iter() {
            match shape {
                Shape::Circle(c) => nodes.push(Box::new(
                    svg::node::element::Circle::new()
                        .set("cx", c.center.x * scale)
                        .set("cy", c.center.y * scale)
                        .set("r", c.radius * scale)
                        .set("fill", fill)
                        .set("stroke", stroke.clone())
                        .set("stroke-width", stroke_width),
                )),
                Shape::Rect(r) => {
                    let bl = r.bl();
                    let size = r.size();
                    nodes.push(Box::new(
                        svg::node::element::Rectangle::new()
                            .set("x", bl.x * scale)
                            .set("y", bl.y * scale)
                            .set("width", size.x * scale)
                            .set("height", size.y * scale)
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
                    data = data.move_to((points[0] * scale).as_tuple());
                    data = points
                        .iter()
                        .skip(1)
                        .fold(data, |data, point| data.line_to((point * scale).as_tuple()));

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
    pub fn write_svg(&self, path: PathBuf, scale: f32) -> Result<()> {
        let document = self.to_svg(scale);
        svg::save(path.to_str().unwrap(), &document)?;
        Ok(())
    }

    /// Creates a new `Layer` from an .svg file.
    ///
    /// Currently this only imports basic `<path d="...">` commands (`M/m`, `L/l`, `H/h`, `V/v`, `Z/z`).
    pub fn new_from_svg(path: &PathBuf) -> Result<Layer> {
        let mut svg_content = String::new();
        let mut layer = Layer::new();
        let mut transform_stack: Vec<SvgTransform> = vec![SvgTransform::identity()];
        let mut svg_units_to_cm_scale: f32 = 1.0;

        for event in svg::open(path, &mut svg_content)? {
            if let Event::Tag(tag, tag_type, attributes) = event {
                if tag == "svg" && !matches!(tag_type, SvgTagType::End) {
                    if let Some(scale) = Self::parse_svg_root_units_to_cm_scale(&attributes) {
                        svg_units_to_cm_scale = scale;
                    }
                }

                let parent_transform = *transform_stack.last().unwrap_or(&SvgTransform::identity());
                let local_transform =
                    Self::parse_svg_transform_attr(&attributes).unwrap_or(SvgTransform::identity());
                let combined_transform = parent_transform.multiply(local_transform);

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
                            for mut path in Self::parse_svg_path_data(path_data.as_str()) {
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

    fn iter_svg_f32_tokens(input: &str) -> impl Iterator<Item = f32> + '_ {
        input
            .split(|c: char| c == ',' || c.is_ascii_whitespace())
            .filter(|token| !token.is_empty())
            .filter_map(|token| token.parse::<f32>().ok())
    }

    fn parse_svg_points_attr(points_attr: &svg::node::Value) -> Vec<V2> {
        let points = points_attr.to_string();
        let mut tokens = Self::iter_svg_f32_tokens(points.as_str());
        let mut parsed_points = Vec::new();

        while let Some(x) = tokens.next() {
            let Some(y) = tokens.next() else {
                break;
            };
            parsed_points.push(V2::new(x, y));
        }

        parsed_points
    }

    fn apply_svg_transform_to_path(path: &Path, transform: SvgTransform, scale_to_cm: f32) -> Path {
        Path::new_from_iter(
            path.get_points_ref()
                .iter()
                .map(|point| transform.apply(*point) * scale_to_cm),
        )
    }

    fn parse_svg_root_units_to_cm_scale(attributes: &Attributes) -> Option<f32> {
        let view_box = attributes.get("viewBox")?.to_string();
        let mut view_box_values = Self::iter_svg_f32_tokens(view_box.as_str());
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

    fn parse_svg_transform_attr(attributes: &Attributes) -> Option<SvgTransform> {
        let transform_attr = attributes.get("transform")?.to_string();
        Self::parse_svg_transform_chain(transform_attr.as_str())
    }

    fn parse_svg_transform_chain(transform_attr: &str) -> Option<SvgTransform> {
        let mut combined = SvgTransform::identity();
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
            let args_str = &rest[(open_idx + 1)..close_idx];
            let mut args = [0.0_f32; 6];
            let mut args_len = 0usize;
            for value in Self::iter_svg_f32_tokens(args_str) {
                if args_len < args.len() {
                    args[args_len] = value;
                }
                args_len += 1;
            }

            let next_transform = match (name, args_len) {
                ("matrix", 6) => Some(SvgTransform::matrix(
                    args[0], args[1], args[2], args[3], args[4], args[5],
                )),
                ("translate", 1) => Some(SvgTransform::translate(args[0], 0.0)),
                ("translate", len) if len >= 2 => Some(SvgTransform::translate(args[0], args[1])),
                ("scale", 1) => Some(SvgTransform::scale(args[0], args[0])),
                ("scale", len) if len >= 2 => Some(SvgTransform::scale(args[0], args[1])),
                _ => {
                    // TODO: Support rotate/skew transforms and malformed transform expressions.
                    None
                }
            };

            if let Some(next_transform) = next_transform {
                combined = combined.multiply(next_transform);
                parsed_any = true;
            }

            rest = rest[(close_idx + 1)..].trim_start();
        }

        if parsed_any {
            Some(combined)
        } else {
            None
        }
    }

    fn parse_svg_path_data(path_data: &str) -> Vec<Path> {
        let mut parsed_paths: Vec<Path> = Vec::new();

        let mut current_points: Vec<V2> = Vec::new();
        let mut current_point: Option<V2> = None;
        let mut subpath_start: Option<V2> = None;

        let mut active_command = 'M';
        let mut command_is_set = false;

        let tokens = Self::tokenize_svg_path_data(path_data);
        let mut token_index = 0;

        while token_index < tokens.len() {
            if let SvgPathToken::Command(command) = tokens[token_index] {
                active_command = command;
                command_is_set = true;
                token_index += 1;
            } else if !command_is_set {
                // TODO: Handle malformed path data where numbers appear before the first command.
                break;
            }

            match active_command {
                'M' | 'm' => {
                    let mut is_first_pair = true;
                    while let Some((x, y)) = Self::read_svg_pair(&tokens, &mut token_index) {
                        let mut next_point = V2::new(x, y);
                        if active_command == 'm' {
                            next_point = current_point.unwrap_or(V2::zero()) + next_point;
                        }

                        if is_first_pair {
                            if current_points.len() >= 2 {
                                parsed_paths
                                    .push(Path::new_from(std::mem::take(&mut current_points)));
                            }
                            current_points.push(next_point);
                            subpath_start = Some(next_point);
                            is_first_pair = false;
                        } else {
                            current_points.push(next_point);
                        }
                        current_point = Some(next_point);
                    }
                }
                'L' | 'l' => {
                    while let Some((x, y)) = Self::read_svg_pair(&tokens, &mut token_index) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        let mut next_point = V2::new(x, y);
                        if active_command == 'l' {
                            next_point = current_point.unwrap_or(V2::zero()) + next_point;
                        }
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'H' | 'h' => {
                    while let Some(x) = Self::read_svg_number(&tokens, &mut token_index) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 'h' {
                            V2::new(base.x + x, base.y)
                        } else {
                            V2::new(x, base.y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'V' | 'v' => {
                    while let Some(y) = Self::read_svg_number(&tokens, &mut token_index) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 'v' {
                            V2::new(base.x, base.y + y)
                        } else {
                            V2::new(base.x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'Z' | 'z' => {
                    if !current_points.is_empty() {
                        if let Some(start) = subpath_start {
                            if current_points.last() != Some(&start) {
                                current_points.push(start);
                            }
                        }

                        if current_points.len() >= 2 {
                            parsed_paths.push(Path::new_from(std::mem::take(&mut current_points)));
                        } else {
                            current_points.clear();
                        }
                        current_point = subpath_start;
                        subpath_start = None;
                    }
                }
                'C' | 'c' => {
                    while let (Some(_x1), Some(_y1), Some(_x2), Some(_y2), Some(x), Some(y)) = (
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                    ) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        // TODO: Sample bezier curves instead of approximating them as straight segments.
                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 'c' {
                            V2::new(base.x + x, base.y + y)
                        } else {
                            V2::new(x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'S' | 's' => {
                    while let (Some(_x2), Some(_y2), Some(x), Some(y)) = (
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                    ) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        // TODO: Sample smooth cubic bezier curves instead of endpoint approximation.
                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 's' {
                            V2::new(base.x + x, base.y + y)
                        } else {
                            V2::new(x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'Q' | 'q' => {
                    while let (Some(_cx), Some(_cy), Some(x), Some(y)) = (
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                    ) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        // TODO: Sample quadratic bezier curves instead of endpoint approximation.
                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 'q' {
                            V2::new(base.x + x, base.y + y)
                        } else {
                            V2::new(x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'T' | 't' => {
                    while let Some((x, y)) = Self::read_svg_pair(&tokens, &mut token_index) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        // TODO: Sample smooth quadratic bezier curves instead of endpoint approximation.
                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 't' {
                            V2::new(base.x + x, base.y + y)
                        } else {
                            V2::new(x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                'A' | 'a' => {
                    while let (
                        Some(_rx),
                        Some(_ry),
                        Some(_x_axis_rotation),
                        Some(_large_arc_flag),
                        Some(_sweep_flag),
                        Some(x),
                        Some(y),
                    ) = (
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                        Self::read_svg_number(&tokens, &mut token_index),
                    ) {
                        Self::ensure_svg_subpath_started(
                            &mut current_points,
                            &mut subpath_start,
                            current_point,
                        );

                        // TODO: Sample elliptical arcs instead of endpoint approximation.
                        let base = current_point.unwrap_or(V2::zero());
                        let next_point = if active_command == 'a' {
                            V2::new(base.x + x, base.y + y)
                        } else {
                            V2::new(x, y)
                        };
                        current_points.push(next_point);
                        current_point = Some(next_point);
                    }
                }
                _ => {
                    // TODO: Support additional SVG path commands.
                    while Self::read_svg_number(&tokens, &mut token_index).is_some() {}
                }
            }
        }

        if current_points.len() >= 2 {
            parsed_paths.push(Path::new_from(current_points));
        }

        parsed_paths
    }

    fn tokenize_svg_path_data(path_data: &str) -> Vec<SvgPathToken> {
        let mut tokens = Vec::new();
        let mut chars = path_data.chars().peekable();

        while let Some(ch) = chars.peek().copied() {
            if ch.is_ascii_alphabetic() {
                chars.next();
                tokens.push(SvgPathToken::Command(ch));
                continue;
            }

            if ch.is_ascii_whitespace() || ch == ',' {
                chars.next();
                continue;
            }

            let mut number = String::new();
            let mut seen_exponent = false;

            while let Some(next_ch) = chars.peek().copied() {
                if next_ch.is_ascii_digit() || next_ch == '.' {
                    number.push(next_ch);
                    chars.next();
                    continue;
                }

                if (next_ch == 'e' || next_ch == 'E') && !seen_exponent {
                    seen_exponent = true;
                    number.push(next_ch);
                    chars.next();
                    continue;
                }

                if next_ch == '-' || next_ch == '+' {
                    if number.is_empty() || number.ends_with('e') || number.ends_with('E') {
                        number.push(next_ch);
                        chars.next();
                        continue;
                    }
                    break;
                }

                break;
            }

            if !number.is_empty() {
                if let std::result::Result::Ok(value) = number.parse::<f32>() {
                    tokens.push(SvgPathToken::Number(value));
                } else {
                    // TODO: Handle parse failures with richer diagnostics (line/column/path id).
                }
            } else {
                chars.next();
            }
        }

        tokens
    }

    fn ensure_svg_subpath_started(
        current_points: &mut Vec<V2>,
        subpath_start: &mut Option<V2>,
        current_point: Option<V2>,
    ) {
        if current_points.is_empty() {
            if let Some(start) = current_point {
                current_points.push(start);
                if subpath_start.is_none() {
                    *subpath_start = Some(start);
                }
            }
        }
    }

    fn read_svg_number(tokens: &[SvgPathToken], token_index: &mut usize) -> Option<f32> {
        if *token_index >= tokens.len() {
            return None;
        }

        match tokens[*token_index] {
            SvgPathToken::Number(value) => {
                *token_index += 1;
                Some(value)
            }
            SvgPathToken::Command(_) => None,
        }
    }

    fn read_svg_pair(tokens: &[SvgPathToken], token_index: &mut usize) -> Option<(f32, f32)> {
        let x = Self::read_svg_number(tokens, token_index)?;
        let y = Self::read_svg_number(tokens, token_index)?;
        Some((x, y))
    }

    /// Returns a new `Layer` with [`Shape`]s that start/end at another [`Shape`]'s start/end combined into a single [`Path`].
    /// This is done until no more shapes can be combined, recursively for all sublayers individually.
    ///
    /// `max_angle_delta` is the maximum angle difference between the start/end points of the two shapes that will still be combined. Set to `None` to ignore angles.
    pub fn combine_shapes_recursive(&self, max_angle_delta: Option<Angle>) -> Self {
        let (combineable, noncombineable) =
            Layer::group_shapes_combineable_noncombineable(self.iter());

        let shapes: Vec<Shape> = self
            .combine_shapes_flat_iterate_until_no_effect(combineable, &max_angle_delta)
            .into_iter()
            .map(|path| path.into())
            .chain(noncombineable)
            .collect();

        let sublayers = self
            .sublayers
            .iter()
            .map(|sublayer| sublayer.combine_shapes_recursive(max_angle_delta))
            .collect();

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Returns a new flattened `Layer` with [`Shape`]s that start/end at another [`Shape`]'s start/end combined into a single [`Path`]. see [`Layer::combine_shapes_recursive`].
    pub fn combine_shapes_flat(&self, max_angle_delta: Option<Angle>) -> Self {
        let (combineable, noncombineable) =
            Layer::group_shapes_combineable_noncombineable(self.iter_flattened());

        let shapes: Vec<Shape> = self
            .combine_shapes_flat_iterate_until_no_effect(combineable, &max_angle_delta)
            .into_iter()
            .map(|path| path.into())
            .chain(noncombineable)
            .collect();

        Layer::new_from(shapes)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }
    fn group_shapes_combineable_noncombineable<'a, I>(shapes: I) -> (Vec<Path>, Vec<Shape>)
    where
        I: IntoIterator<Item = &'a Shape>,
    {
        let shape_iter = shapes.into_iter();
        let (shape_count_lower_bound, _) = shape_iter.size_hint();
        let mut combineable = Vec::with_capacity(shape_count_lower_bound);
        let mut noncombineable = Vec::with_capacity(shape_count_lower_bound);

        for shape in shape_iter {
            match shape {
                Shape::Path(path) => {
                    if path.get_points_ref().len() <= 1 {
                        continue;
                    }
                    combineable.push(path.clone());
                }
                Shape::Circle(_) | Shape::Rect(_) => {
                    noncombineable.push(shape.clone());
                }
            }
        }
        (combineable, noncombineable)
    }
    fn combine_shapes_flat_iterate_until_no_effect(
        &self,
        mut paths: Vec<Path>,
        max_angle_delta: &Option<Angle>,
    ) -> Vec<Path> {
        let mut last_num_paths = paths.len();
        loop {
            paths = self.combine_shapes_flat_iterate(&paths, max_angle_delta);
            if last_num_paths <= paths.len() || paths.len() == 1 {
                return paths;
            }
            last_num_paths = paths.len();
        }
    }
    fn prepend_paths(path_candidate: &Path, reverse_candidate: bool, current_path: &Path) -> Path {
        let candidate_points = path_candidate.get_points_ref();
        let current_points = current_path.get_points_ref();
        let mut points =
            Vec::with_capacity(candidate_points.len() + current_points.len().saturating_sub(1));

        if reverse_candidate {
            points.extend(candidate_points.iter().rev().copied());
        } else {
            points.extend(candidate_points.iter().copied());
        }
        points.extend(current_points.iter().skip(1).copied());

        Path::new_from(points)
    }

    fn combine_shapes_flat_iterate(
        &self,
        paths: &[Path],
        max_angle_delta: &Option<Angle>,
    ) -> Vec<Path> {
        let mut combined_paths = Vec::new();

        // iterate paths, combine
        let mut mask = vec![false; paths.len()];
        let mut current_path = Path::new();
        let mut current_path_start = PathEnd {
            point: V2::new(0.0, 0.0),
            angle: Angle::zero(),
        };
        let mut current_path_end = PathEnd {
            point: V2::new(0.0, 0.0),
            angle: Angle::zero(),
        };

        for (i, path) in paths.iter().enumerate() {
            if mask[i] {
                continue;
            }

            // start new path
            if current_path.is_empty() {
                current_path.push_many(path.get_points_ref());
                current_path_start = PathEnd::from_path_start(path);
                current_path_end = PathEnd::from_path_end(path);
                mask[i] = true;
            }

            for (j, path_candidate) in paths.iter().enumerate().skip(i + 1) {
                if mask[j] {
                    continue;
                }

                let start = PathEnd::from_path_start(path_candidate);
                let end = PathEnd::from_path_end(path_candidate);

                if current_path_end.is_compatible(&start, max_angle_delta) {
                    // regular append
                    // current_start -> current_end -> # -> start -> end
                    current_path.push_iter_ref(path_candidate.get_points_ref().iter().skip(1));

                    current_path_end = end;
                    mask[j] = true;
                } else if current_path_end.is_compatible(&end.flipped(), max_angle_delta) {
                    // reverse candidate and append
                    // current_start -> current_end -> # -> end -> start
                    current_path
                        .push_iter_ref(path_candidate.get_points_ref().iter().rev().skip(1));

                    current_path_end = start.flipped();
                    mask[j] = true;
                } else if current_path_start.is_compatible(&end, max_angle_delta) {
                    // regular prepend
                    // start -> end -> # -> current_start -> current_end
                    current_path = Self::prepend_paths(path_candidate, false, &current_path);

                    current_path_start = start;
                    mask[j] = true;
                } else if current_path_start.is_compatible(&start.flipped(), max_angle_delta) {
                    // reverse candidate and prepend
                    // end -> start -> # -> current_start -> current_end
                    current_path = Self::prepend_paths(path_candidate, true, &current_path);

                    current_path_start = end.flipped();
                    mask[j] = true;
                }
            }

            if !current_path.is_empty() {
                combined_paths.push(current_path);
                current_path = Path::new();
            }
        }

        combined_paths
    }

    /// Map a function recursively to all [`Shape`]s in the `Layer` and its sublayers.
    pub fn map_recursive_mut<F: Fn(&mut Shape)>(&mut self, f: F) {
        self.map_recursive_mut_internal(&f)
    }
    fn map_recursive_mut_internal<F: Fn(&mut Shape)>(&mut self, f: &F) {
        for shape in &mut self.shapes {
            f(shape);
        }
        for sublayer in &mut self.sublayers {
            sublayer.map_recursive_mut_internal(f);
        }
    }

    /// Create a new [`Layer`] with a function mapped recursively to all [`Shape`]s in the `Layer` and its sublayers.
    pub fn map_recursive<F>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> Shape + Send + Sync,
    {
        let f = Arc::new(f);
        self.map_shapes_recursive_internal(f)
    }
    fn map_shapes_recursive_internal<F>(&self, f: Arc<F>) -> Self
    where
        F: Fn(&Shape) -> Shape + Send + Sync,
    {
        let (shapes, sublayers) = rayon::join(
            || self.shapes.par_iter().map(|shape| f(shape)).collect(),
            || {
                self.sublayers
                    .par_iter()
                    .map(|sublayer| sublayer.map_shapes_recursive_internal(f.clone()))
                    .collect()
            },
        );

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Filter the [`Shape`]s in the `Layer` and its sublayers with a predicate function.
    pub fn filter_recursive_mut<F>(&mut self, predicate: F)
    where
        F: Fn(&Shape) -> bool,
    {
        self.filter_recursive_mut_internal(&predicate)
    }
    fn filter_recursive_mut_internal<F>(&mut self, predicate: &F)
    where
        F: Fn(&Shape) -> bool,
    {
        self.shapes.retain(|shape| predicate(shape));
        for sublayer in &mut self.sublayers {
            sublayer.filter_recursive_mut_internal(predicate);
        }
    }

    /// Create a new [`Layer`] with the [`Shape`]s in the `Layer` and its sublayers filtered with a predicate function.
    pub fn filter_recursive<F>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> bool + Clone + Send + Sync,
    {
        let f = Arc::new(f);
        self.filter_recursive_internal(f)
    }
    fn filter_recursive_internal<F>(&self, f: Arc<F>) -> Self
    where
        F: Fn(&Shape) -> bool + Send + Sync,
    {
        let (filtered_shapes, filtered_sublayers) = rayon::join(
            || {
                self.shapes
                    .par_iter()
                    .filter(|shape| f(shape))
                    .cloned()
                    .collect()
            },
            || {
                self.sublayers
                    .par_iter()
                    .map(|layer| layer.filter_recursive_internal(f.clone()))
                    .collect()
            },
        );

        Layer::new_from_shapes_and_layers(filtered_shapes, filtered_sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Returns a [`Masked`] object all [`Shape`]s of the `Layer` and its sublayers, grouping parts of all [`Shape`]s into `inside` and `outside`.
    ///
    /// This is done using crate [`geo`]'s [`geo::algorithm::bool_ops::BooleanOps::clip`].
    /// see [`Shape::mask_geo`].
    ///
    /// There can be unexpected outputs for example with shapes that self-intersect.
    /// See also [`Layer::mask_flattened_brute_force`] for a different and more stable masking.
    pub fn mask_geo_flattened(&self, mask: &Shape, sample_settings: SampleSettings) -> Masked {
        let shapes: Vec<_> = self.iter_flattened().collect();
        let masked_shapes: Vec<_> = shapes
            .par_iter()
            .map(|shape| shape.mask_geo(mask, sample_settings))
            .collect();

        let mut inside = Layer::with_capacity(masked_shapes.len() * 2);
        let mut outside = Layer::with_capacity(masked_shapes.len() * 2);

        for masked in masked_shapes {
            inside.push_layer_flat(masked.inside);
            outside.push_layer_flat(masked.outside);
        }

        Masked { inside, outside }
    }

    /// see [`Layer::mask_geo_flattened`].
    pub fn mask_geo_flattened_inside(
        &self,
        mask: &Shape,
        sample_settings: SampleSettings,
    ) -> Layer {
        self.iter_flattened()
            .map(|shape| shape.mask_geo_inside(mask, sample_settings))
            .collect()
    }

    /// see [`Layer::mask_geo_flattened`].
    pub fn mask_geo_flattened_outside(
        &self,
        mask: &Shape,
        sample_settings: SampleSettings,
    ) -> Layer {
        self.iter_flattened()
            .map(|shape| shape.mask_geo_outside(mask, sample_settings))
            .collect()
    }

    /// Reduces the number of points for all [`Shape`]s in this `Layer` and its sublayers. see [`Shape::reduce_points`].
    pub fn reduce_points_recursive(&self, aggression_factor: f32) -> Self {
        self.map_recursive(|shape| shape.reduce_points(aggression_factor))
    }

    /// Returns a [`Masked`] object all [`Shape`]s of the `Layer` and its sublayers, grouping parts of all [`Shape`]s into `inside` and `outside`.
    ///
    /// This is done by oversampling the [`Shape`] and checking for each point if it is inside or outside the mask using [`Shape::contains_point`].
    ///
    /// see [`Shape::mask_brute_force`].
    ///
    /// see also [`Layer::mask_geo_flattened`].
    pub fn mask_flattened_brute_force(
        &self,
        mask: &Shape,
        sample_settings: SampleSettings,
    ) -> Masked {
        let mut inside = Layer::new()
            .with_props(self.props.clone())
            .with_props_inheritable(self.props_inheritable.clone());
        let mut outside = Layer::new()
            .with_props(self.props.clone())
            .with_props_inheritable(self.props_inheritable.clone());

        for shape in self.iter_flattened() {
            let masked = shape.mask_brute_force(mask, sample_settings);
            inside.push_layer_flat(masked.inside);
            outside.push_layer_flat(masked.outside);
        }

        Masked { inside, outside }
    }

    /// Returns a new `Layer` with all of this layers [`Shape`]s (non-recursive) ordered in a way that they can be plotted with a minimum of pen travel between shapes.
    /// This is done with a greedy algorithm that always chooses the closest shape to the current position.
    ///
    /// see also [`Layer::optimize_recursive`].
    fn optimize_shape_start_end(shape: &Shape) -> (V2, V2) {
        match shape {
            Shape::Path(path) => {
                let points = path.get_points_ref();
                if let (Some(start), Some(end)) = (points.first(), points.last()) {
                    (*start, *end)
                } else {
                    (V2::zero(), V2::zero())
                }
            }
            Shape::Circle(circle) => {
                let start = circle.center + V2::new(circle.radius, 0.0);
                (start, start)
            }
            Shape::Rect(rect) => {
                let start = rect.bl();
                (start, start)
            }
        }
    }

    pub fn optimize(&self) -> Self {
        let starts_and_ends: Vec<_> = self
            .shapes
            .iter()
            .map(Self::optimize_shape_start_end)
            .collect();

        let mut unused_items_indices: BTreeSet<usize> = (0..self.shapes.len()).collect();

        let mut pos = V2::zero();
        let mut optimized = Layer::with_capacity(self.shapes.len())
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone());

        while !unused_items_indices.is_empty() {
            let mut best_distance = f32::INFINITY;
            let mut best_index = 0;
            let mut reversed = false;

            for unused_i in unused_items_indices.iter() {
                let dist_to_start = starts_and_ends[*unused_i].0.dist_squared(pos);
                let dist_to_end = starts_and_ends[*unused_i].1.dist_squared(pos);

                if dist_to_start < best_distance {
                    reversed = true;
                    best_distance = dist_to_start;
                    best_index = *unused_i;
                }
                if dist_to_end < best_distance {
                    reversed = false;
                    best_distance = dist_to_end;
                    best_index = *unused_i;
                }
            }

            optimized.push(self.shapes[best_index].clone());
            if reversed {
                pos = starts_and_ends[best_index].0;
            } else {
                pos = starts_and_ends[best_index].1;
            }
            unused_items_indices.remove(&best_index);
        }

        optimized
    }

    /// Applies the same optimization like [`Layer::optimize`], recursively for all sublayers.
    pub fn optimize_recursive(&self) -> Self {
        let mut optimized = self.optimize();
        for sublayer in &self.sublayers {
            optimized.push_layer(sublayer.optimize_recursive());
        }
        optimized
    }

    /// Returns a new `Layer` without any sublayers direclty containing all [`Shape`]s of this `Layer` and its sublayers.
    ///
    /// see also [`Layer::iter_flattened`].
    pub fn flatten(&self) -> Self {
        Layer::new_from_shapes_and_layers(self.iter_flattened().cloned().collect(), Vec::new())
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Shape>> for Layer {
    fn from(shapes: Vec<Shape>) -> Self {
        Layer::new_from(shapes)
    }
}

impl From<Vec<&Shape>> for Layer {
    fn from(shapes: Vec<&Shape>) -> Self {
        Layer::new_from(shapes.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Path>> for Layer {
    fn from(paths: Vec<Path>) -> Self {
        Layer::new_from(paths.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<&Path>> for Layer {
    fn from(paths: Vec<&Path>) -> Self {
        Layer::new_from(paths.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Vec<V2>>> for Layer {
    fn from(points_lists: Vec<Vec<V2>>) -> Self {
        points_lists.into_iter().collect()
    }
}

impl From<Vec<Circle>> for Layer {
    fn from(circles: Vec<Circle>) -> Self {
        Layer::new_from(circles.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<&Circle>> for Layer {
    fn from(circles: Vec<&Circle>) -> Self {
        Layer::new_from(circles.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Rect>> for Layer {
    fn from(rects: Vec<Rect>) -> Self {
        Layer::new_from(rects.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<&Rect>> for Layer {
    fn from(rects: Vec<&Rect>) -> Self {
        Layer::new_from(rects.into_iter().map(Into::into).collect())
    }
}

impl From<Vec<Layer>> for Layer {
    fn from(layers: Vec<Layer>) -> Self {
        Layer::new_from_shapes_and_layers(Vec::new(), layers)
    }
}

impl From<Vec<&Layer>> for Layer {
    fn from(layers: Vec<&Layer>) -> Self {
        Layer::new_from_shapes_and_layers(Vec::new(), layers.into_iter().cloned().collect())
    }
}

#[derive(Debug, Clone)]
pub struct LayerFlattenedIterator<'a> {
    stack: Vec<&'a Layer>,
    current_layer_iterator: Option<std::slice::Iter<'a, Shape>>,
}

impl<'a> LayerFlattenedIterator<'a> {
    fn new(layer: &'a Layer) -> LayerFlattenedIterator<'a> {
        Self {
            stack: vec![layer],
            current_layer_iterator: None,
        }
    }
}

impl<'a> Iterator for LayerFlattenedIterator<'a> {
    type Item = &'a Shape;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current_layer_iter) = &mut self.current_layer_iterator {
                if let Some(shape) = current_layer_iter.next() {
                    return Some(shape);
                }
            }

            if let Some(layer) = self.stack.pop() {
                self.current_layer_iterator = Some(layer.shapes.iter());
                for sublayer in layer.sublayers.iter().rev() {
                    self.stack.push(sublayer);
                }
            } else {
                return None;
            }
        }
    }
}

impl<S> FromIterator<S> for Layer
where
    S: Into<Shape>,
{
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        Layer {
            shapes: iter.into_iter().map(Into::into).collect(),
            sublayers: Vec::new(),
            props_inheritable: Inheritable::Inherit,
            props: LayerProps::default(),
        }
    }
}

impl FromIterator<Layer> for Layer {
    fn from_iter<I: IntoIterator<Item = Layer>>(iter: I) -> Self {
        Layer::new_from_shapes_and_layers(Vec::new(), iter.into_iter().collect())
    }
}

impl Translate for Layer {
    fn translate(&self, dist: V2) -> Self {
        self.map_recursive(|shape| shape.translate(dist))
    }
    fn translate_mut(&mut self, dist: V2) {
        self.map_recursive_mut(|shape| shape.translate_mut(dist));
    }
}

impl Rotate for Layer {
    fn rotate(&self, angle: Angle) -> Self {
        self.map_recursive(|shape| shape.rotate(angle))
    }
    fn rotate_mut(&mut self, angle: Angle) {
        self.map_recursive_mut(|shape| shape.rotate_mut(angle));
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.map_recursive(|shape| shape.rotate_around(pivot, angle))
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        self.map_recursive_mut(|shape| shape.rotate_around_mut(pivot, angle));
    }
}

impl Scale for Layer {
    fn scale(&self, factor: f32) -> Self {
        self.map_recursive(|shape| shape.scale(factor))
    }

    fn scale_mut(&mut self, factor: f32) {
        self.map_recursive_mut(|shape| shape.scale_mut(factor));
    }
}

impl Scale2D for Layer {
    fn scale_2d(&self, factor: V2) -> Self {
        self.map_recursive(|shape| shape.scale_2d(factor))
    }

    fn scale_2d_mut(&mut self, factor: V2) {
        self.map_recursive_mut(|shape| shape.scale_2d_mut(factor));
    }
}

impl Normalize for Layer {}

impl Mirror for Layer {
    fn mirror_x(&self) -> Self {
        self.map_recursive(|shape| shape.mirror_x())
    }

    fn mirror_x_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.mirror_x_mut());
    }

    fn mirror_y(&self) -> Self {
        self.map_recursive(|shape| shape.mirror_y())
    }

    fn mirror_y_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.mirror_y_mut());
    }
}

impl BoundingBox for Layer {
    fn bounding_box(&self) -> Option<Rect> {
        let mut min = None;
        let mut max = None;
        for shape in self.iter_flattened() {
            if let Some(bb) = shape.bounding_box() {
                if min.is_none() {
                    min = Some(bb.bl());
                } else {
                    min = Some(min.unwrap().min(bb.bl()));
                }
                if max.is_none() {
                    max = Some(bb.tr());
                } else {
                    max = Some(max.unwrap().max(bb.tr()));
                }
            }
        }
        if min.is_none() || max.is_none() {
            return None;
        }
        Some(Rect::new(min.unwrap(), max.unwrap()))
    }
}
