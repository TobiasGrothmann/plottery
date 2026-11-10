use anyhow::{Ok, Result};
use bincode::{deserialize_from, serialize};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{fs::File, io::Write, iter::FromIterator, path::PathBuf, vec};
use svg::{
    node::{
        element::{path::Data, tag::Type as SvgTagType, Group},
        Attributes,
    },
    parser::Event,
    Document, Node,
};

use crate::{
    geometry::TransformMatrix,
    traits::{Fit, Scale, Scale2D, Transform, Translate},
    Angle, Blur, BoundingBox, Hatch, Circle, Masked, Mirror, Path, Plottable, Rect, Rotate, Rotate90,
    SampleSettings, Shape, SpatialGrid, V2,
};

use super::{
    combine_paths::combine_paths,
    svg_path_data::parse_svg_path_data,
    svg_transform::{iter_svg_f32_tokens, parse_svg_transform_attr},
    ColorRgb, Inheritable, LayerProps, LayerPropsInheritable,
};

/// SVG exports use millimetres as user units.
const SVG_UNITS_PER_CM: f32 = 10.0;

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
    /// Creates a new `Layer` from anything that yields shapes.
    pub fn new_from<I>(shapes: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Shape>,
    {
        Self {
            shapes: shapes.into_iter().map(Into::into).collect(),
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
    #[must_use]
    pub fn with_props_inheritable(mut self, props: Inheritable<LayerPropsInheritable>) -> Self {
        self.set_props_inheritable(props);
        self
    }
    /// set the non-inheritable properties of the layer
    pub fn set_props(&mut self, props: LayerProps) {
        self.props = props;
    }
    /// set the non-inheritable properties of the layer
    #[must_use]
    pub fn with_props(mut self, props: LayerProps) -> Self {
        self.set_props(props);
        self
    }

    /// Helper to set the color of the layer. see [`Layer::with_props_inheritable`].
    #[must_use]
    pub fn with_color(mut self, color: ColorRgb) -> Self {
        self.props_inheritable = self
            .props_inheritable
            .overwrite_with(&Inheritable::Specified(
                LayerPropsInheritable::inherit_all().with_color(color),
            ));
        self
    }
    /// Helper to set the pen width of the layer. see [`Layer::with_props_inheritable`].
    #[must_use]
    pub fn with_pen_width_cm(mut self, pen_width_cm: f32) -> Self {
        self.props_inheritable = self
            .props_inheritable
            .overwrite_with(&Inheritable::Specified(
                LayerPropsInheritable::inherit_all().with_pen_width_cm(pen_width_cm),
            ));
        self
    }
    /// Helper to set the name of the layer. see [`Layer::with_props`].
    #[must_use]
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
        self.shapes.reserve(layer.len());
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

    /// Iterates all shapes, recursively through all sublayers. Use `.shapes.iter()` for this layer only.
    pub fn iter(&self) -> LayerFlattenedIterator<'_> {
        LayerFlattenedIterator::new(self)
    }
    /// Iterates all [`Path`]s, recursively through all sublayers, skipping other shapes.
    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.iter().filter_map(|shape| match shape {
            Shape::Path(path) => Some(path),
            _ => None,
        })
    }

    /// Number of shapes, recursively through all sublayers. Use `.shapes.len()` for this layer only.
    pub fn len(&self) -> usize {
        self.sublayers
            .iter()
            .fold(self.shapes.len(), |acc, sublayer| acc + sublayer.len())
    }
    /// Returns true if the layer has no [`Shape]s and no sublayers.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty() && self.sublayers.is_empty()
    }

    /// Return the layer as a real-size [`svg::Document`] in mm, y-axis flipped, sublayers as `<g>`.
    pub fn to_svg(&self) -> Document {
        let Some(bounding_box) = self.bounding_box() else {
            return Document::new();
        };
        let height = bounding_box.size().only_y();
        let prepared = self.map_recursive(|shape| shape.mirror_y().translate(height));

        // The view box follows the content so shapes below or left of the origin stay visible.
        let flipped_bl = V2::new(bounding_box.bl().x, height.y - bounding_box.tr().y);
        let origin: V2 = flipped_bl * SVG_UNITS_PER_CM;
        let size: V2 = bounding_box.size() * SVG_UNITS_PER_CM;

        Document::new()
            .set("viewBox", (origin.x, origin.y, size.x, size.y))
            .set("width", format!("{}mm", size.x))
            .set("height", format!("{}mm", size.y))
            // Round like a real pen tip; the default miter join spikes at sharp corners.
            .set("stroke-linejoin", "round")
            .set("stroke-linecap", "round")
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

    /// Returns a new `Layer` with [`Shape`]s that start/end at another [`Shape`]'s start/end combined into a single [`Path`].
    /// Recursive for all sublayers individually.
    ///
    /// `max_angle_delta` is the maximum angle difference between the start/end points of the two shapes that will still be combined. Set to `None` to ignore angles.
    #[must_use]
    pub fn combine_shapes_recursive(&self, max_angle_delta: Option<Angle>) -> Self {
        let (combineable, noncombineable) =
            Layer::group_shapes_combineable_noncombineable(self.shapes.iter());

        let shapes: Vec<Shape> = combine_paths(combineable, max_angle_delta)
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
    #[must_use]
    pub fn combine_shapes_flat(&self, max_angle_delta: Option<Angle>) -> Self {
        let (combineable, noncombineable) =
            Layer::group_shapes_combineable_noncombineable(self.iter());

        let shapes: Vec<Shape> = combine_paths(combineable, max_angle_delta)
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
    #[must_use]
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

    /// Like [`Layer::map_recursive`], but `f` may return any number of shapes (or a whole [`Layer`]) per shape.
    #[must_use]
    pub fn flat_map_recursive<F, I>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> I + Send + Sync,
        I: IntoIterator,
        I::Item: Into<Shape>,
    {
        let f = Arc::new(f);
        self.flat_map_recursive_internal(f)
    }
    fn flat_map_recursive_internal<F, I>(&self, f: Arc<F>) -> Self
    where
        F: Fn(&Shape) -> I + Send + Sync,
        I: IntoIterator,
        I::Item: Into<Shape>,
    {
        let shapes: Vec<Shape> = self
            .shapes
            .iter()
            .flat_map(|shape| f(shape).into_iter().map(Into::into))
            .collect();
        let sublayers = self
            .sublayers
            .par_iter()
            .map(|layer| layer.flat_map_recursive_internal(f.clone()))
            .collect();

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Oversamples every shape and moves each point with `f`, recursively. see [`Shape::distort`].
    #[must_use]
    pub fn distort<F>(&self, sample_settings: SampleSettings, f: F) -> Self
    where
        F: Fn(V2) -> V2 + Send + Sync,
    {
        self.map_recursive(|shape| shape.distort(sample_settings, &f).into())
    }

    /// Dashes every shape, recursively. see [`Shape::dash`].
    #[must_use]
    pub fn dash(&self, dash_length: f32, gap_length: f32, sample_settings: SampleSettings) -> Self {
        self.flat_map_recursive(|shape| shape.dash(dash_length, gap_length, sample_settings))
    }

    /// Dashes every shape with a custom dash function, recursively. see [`Shape::dash_fn`].
    #[must_use]
    pub fn dash_fn<F>(&self, sample_settings: SampleSettings, f: F) -> Self
    where
        F: Fn(V2, bool, f32, f32) -> f32 + Send + Sync,
    {
        self.flat_map_recursive(|shape| shape.dash_fn(sample_settings, &f))
    }

    /// Blurs every shape, keeping sublayers and props. see [`Blur`].
    #[must_use]
    pub fn blur(&self, blur: &Blur) -> Self {
        blur.blur_layer(self)
    }

    /// Fills all closed shapes as one even-odd region, so holes work and overlaps cancel out. Open shapes are skipped, see [`Hatch`].
    #[must_use]
    pub fn hatch(&self, hatch: &Hatch) -> Self {
        hatch.hatch_layer(self)
    }

    /// Create a new [`Layer`] with the [`Shape`]s in the `Layer` and its sublayers filtered with a predicate function.
    #[must_use]
    pub fn filter_recursive<F>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> bool + Send + Sync,
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
    pub fn mask_geo_flattened<M: Into<Shape>>(
        &self,
        mask: M,
        sample_settings: SampleSettings,
    ) -> Masked {
        let mask: Shape = mask.into();
        let shapes: Vec<_> = self.iter().collect();
        let masked_shapes: Vec<_> = shapes
            .par_iter()
            .map(|shape| shape.mask_geo(&mask, sample_settings))
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
    pub fn mask_geo_flattened_inside<M: Into<Shape>>(
        &self,
        mask: M,
        sample_settings: SampleSettings,
    ) -> Layer {
        let mask: Shape = mask.into();
        self.iter()
            .map(|shape| shape.mask_geo_inside(&mask, sample_settings))
            .collect()
    }

    /// see [`Layer::mask_geo_flattened`].
    pub fn mask_geo_flattened_outside<M: Into<Shape>>(
        &self,
        mask: M,
        sample_settings: SampleSettings,
    ) -> Layer {
        let mask: Shape = mask.into();
        self.iter()
            .map(|shape| shape.mask_geo_outside(&mask, sample_settings))
            .collect()
    }

    /// Reduces the number of points for all [`Shape`]s in this `Layer` and its sublayers. see [`Shape::reduce_points`].
    #[must_use]
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
    pub fn mask_flattened_brute_force<M: Into<Shape>>(
        &self,
        mask: M,
        sample_settings: SampleSettings,
    ) -> Masked {
        let mask: Shape = mask.into();
        let mut inside = Layer::new()
            .with_props(self.props.clone())
            .with_props_inheritable(self.props_inheritable.clone());
        let mut outside = Layer::new()
            .with_props(self.props.clone())
            .with_props_inheritable(self.props_inheritable.clone());

        for shape in self.iter() {
            let masked = shape.mask_brute_force(&mask, sample_settings);
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

    #[must_use]
    pub fn optimize(&self) -> Self {
        let starts_and_ends: Vec<_> = self
            .shapes
            .iter()
            .map(Self::optimize_shape_start_end)
            .collect();

        // each shape is in the grid twice, at its start and at its end
        let mut ends: SpatialGrid<(usize, bool)> = starts_and_ends
            .iter()
            .enumerate()
            .flat_map(|(i, &(start, end))| [(start, (i, true)), (end, (i, false))])
            .collect();

        let mut pos = V2::zero();
        let shapes = std::iter::from_fn(|| {
            let (_, &(index, entered_at_start)) = ends.nearest(pos)?;
            let (start, end) = starts_and_ends[index];
            ends.remove(start, &(index, true));
            ends.remove(end, &(index, false));
            // the pen leaves from the end it did not enter at
            pos = if entered_at_start { end } else { start };
            Some(self.shapes[index].clone())
        });
        Layer::new_from(shapes)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Applies the same optimization like [`Layer::optimize`], recursively for all sublayers.
    #[must_use]
    pub fn optimize_recursive(&self) -> Self {
        let mut optimized = self.optimize();
        for sublayer in &self.sublayers {
            optimized.push_layer(sublayer.optimize_recursive());
        }
        optimized
    }

    /// Returns a new `Layer` without any sublayers direclty containing all [`Shape`]s of this `Layer` and its sublayers.
    ///
    /// see also [`Layer::iter`].
    #[must_use]
    pub fn flatten(&self) -> Self {
        Layer::new_from_shapes_and_layers(self.iter().cloned().collect(), Vec::new())
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
        Layer::new_from(shapes)
    }
}

impl From<Vec<Path>> for Layer {
    fn from(paths: Vec<Path>) -> Self {
        Layer::new_from(paths)
    }
}

impl From<Vec<&Path>> for Layer {
    fn from(paths: Vec<&Path>) -> Self {
        Layer::new_from(paths)
    }
}

impl From<Vec<Vec<V2>>> for Layer {
    fn from(points_lists: Vec<Vec<V2>>) -> Self {
        points_lists.into_iter().collect()
    }
}

impl From<Vec<Circle>> for Layer {
    fn from(circles: Vec<Circle>) -> Self {
        Layer::new_from(circles)
    }
}

impl From<Vec<&Circle>> for Layer {
    fn from(circles: Vec<&Circle>) -> Self {
        Layer::new_from(circles)
    }
}

impl From<Vec<Rect>> for Layer {
    fn from(rects: Vec<Rect>) -> Self {
        Layer::new_from(rects)
    }
}

impl From<Vec<&Rect>> for Layer {
    fn from(rects: Vec<&Rect>) -> Self {
        Layer::new_from(rects)
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

impl Fit for Layer {}

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
        for shape in self.iter() {
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

impl<'a> IntoIterator for &'a Layer {
    type Item = &'a Shape;
    type IntoIter = LayerFlattenedIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Layer {
    type Item = Shape;
    type IntoIter = std::vec::IntoIter<Shape>;

    /// Iterates all shapes, recursively through all sublayers.
    fn into_iter(self) -> Self::IntoIter {
        let mut shapes = Vec::with_capacity(self.len());
        Layer::append_flattened_shapes_owned(self, &mut shapes);
        shapes.into_iter()
    }
}

impl<S: Into<Shape>> Extend<S> for Layer {
    fn extend<I: IntoIterator<Item = S>>(&mut self, iter: I) {
        self.push_many(iter);
    }
}

impl Extend<Layer> for Layer {
    fn extend<I: IntoIterator<Item = Layer>>(&mut self, iter: I) {
        self.sublayers.extend(iter);
    }
}

impl Rotate90 for Layer {
    fn rotate_90(&self) -> Self {
        self.map_recursive(|shape| shape.rotate_90())
    }
    fn rotate_90_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_90_mut());
    }
    fn rotate_180(&self) -> Self {
        self.map_recursive(|shape| shape.rotate_180())
    }
    fn rotate_180_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_180_mut());
    }
    fn rotate_270(&self) -> Self {
        self.map_recursive(|shape| shape.rotate_270())
    }
    fn rotate_270_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_270_mut());
    }
    fn rotate_90_around(&self, pivot: V2) -> Self {
        self.map_recursive(|shape| shape.rotate_90_around(pivot))
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_90_around_mut(pivot));
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        self.map_recursive(|shape| shape.rotate_180_around(pivot))
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_180_around_mut(pivot));
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        self.map_recursive(|shape| shape.rotate_270_around(pivot))
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_270_around_mut(pivot));
    }
}

impl Transform for Layer {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        self.map_recursive(|shape| shape.transform(matrix))
    }
    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        self.map_recursive_mut(|shape| shape.transform_mut(matrix));
    }
}
