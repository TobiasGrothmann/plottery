use anyhow::{bail, Context, Ok, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{iter::FromIterator, path::PathBuf, vec};

use crate::{
    geometry::TransformMatrix,
    traits::{Fit, Scale, Scale2D, Transform, Translate},
    Angle, Blur, BoundingBox, Circle, Hatch, Masked, Mirror, Path, Plottable, Rect, Rotate,
    Rotate90, SampleSettings, Shape, SpatialGrid, V2,
};

use super::{
    combine_paths::combine_paths,
    file_header::{split_file_header, with_file_header},
    ColorRgb, Inheritable, LayerProps, LayerPropsInheritable,
};

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
    const FILE_MAGIC: [u8; 4] = *b"PLTL";
    /// Changes with every change to the postcard layout of `Layer`.
    const FILE_VERSION: u16 = 2;

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
    /// Reads a `.plotl` file. see [`Layer::write_file`].
    pub fn new_from_file(path: &PathBuf) -> Result<Layer> {
        std::fs::read(path)
            .map_err(anyhow::Error::from)
            .and_then(|bytes| Layer::new_from_binary(&bytes))
            .with_context(|| format!("Failed to read layer '{}'", path.display()))
    }
    /// Writes the `Layer` as a `.plotl` file. see [`Layer::new_from_file`].
    pub fn write_file(&self, path: &PathBuf) -> Result<()> {
        std::fs::write(path, self.to_binary()?)
            .with_context(|| format!("Failed to write layer '{}'", path.display()))
    }
    /// Reads the `.plotl` format written by [`Layer::to_binary`]. There is no migration between its versions.
    pub fn new_from_binary(binary_data: &[u8]) -> Result<Layer> {
        match split_file_header(&Self::FILE_MAGIC, binary_data) {
            Some((Self::FILE_VERSION, payload)) => Ok(postcard::from_bytes(payload)?),
            _ => bail!(
                "The layer is from another Plottery version and can't be read, render it again."
            ),
        }
    }
    /// The `.plotl` format: a file header followed by the postcard of the `Layer`.
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        Ok(with_file_header(
            &Self::FILE_MAGIC,
            Self::FILE_VERSION,
            &postcard::to_stdvec(self)?,
        ))
    }

    #[must_use]
    pub fn with_props_inheritable(mut self, props: Inheritable<LayerPropsInheritable>) -> Self {
        self.props_inheritable = props;
        self
    }
    #[must_use]
    pub fn with_props(mut self, props: LayerProps) -> Self {
        self.props = props;
        self
    }

    /// Helper to set the color of the layer. see [`Layer::with_props_inheritable`].
    #[must_use]
    pub fn with_color(mut self, color: ColorRgb) -> Self {
        self.props_inheritable =
            Inheritable::Specified(self.own_props_inheritable().with_color(color));
        self
    }
    /// Helper to set the pen width of the layer. see [`Layer::with_props_inheritable`].
    #[must_use]
    pub fn with_pen_width(mut self, pen_width: f32) -> Self {
        self.props_inheritable =
            Inheritable::Specified(self.own_props_inheritable().with_pen_width(pen_width));
        self
    }
    /// This layer's inheritable props field by field, all `Inherit` if none are set.
    fn own_props_inheritable(&self) -> LayerPropsInheritable {
        LayerPropsInheritable::inherit_all().overwrite_with(&self.props_inheritable)
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
    /// True if neither this layer nor any sublayer has a [`Shape`], i.e. `len() == 0`.
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty() && self.sublayers.iter().all(Layer::is_empty)
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
                    if path.as_slice().len() <= 1 {
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
    pub fn map_recursive<F: Fn(&Shape) -> Shape>(&self, f: F) -> Self {
        self.map_recursive_internal(&f)
    }
    fn map_recursive_internal<F: Fn(&Shape) -> Shape>(&self, f: &F) -> Self {
        Layer::new_from_shapes_and_layers(
            self.shapes.iter().map(f).collect(),
            self.sublayers
                .iter()
                .map(|sublayer| sublayer.map_recursive_internal(f))
                .collect(),
        )
        .with_props_inheritable(self.props_inheritable.clone())
        .with_props(self.props.clone())
    }

    /// Parallel [`Layer::map_recursive`], only for closures that can't reach any rng.
    pub(crate) fn map_recursive_par<F>(&self, f: F) -> Self
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
        F: Fn(&Shape) -> I,
        I: IntoIterator,
        I::Item: Into<Shape>,
    {
        self.flat_map_recursive_internal(&f)
    }
    fn flat_map_recursive_internal<F, I>(&self, f: &F) -> Self
    where
        F: Fn(&Shape) -> I,
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
            .iter()
            .map(|layer| layer.flat_map_recursive_internal(f))
            .collect();

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Parallel [`Layer::flat_map_recursive`], only for closures that can't reach any rng.
    fn flat_map_recursive_par<F, I>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> I + Send + Sync,
        I: IntoIterator,
        I::Item: Into<Shape>,
    {
        self.flat_map_recursive_par_internal(Arc::new(f))
    }
    fn flat_map_recursive_par_internal<F, I>(&self, f: Arc<F>) -> Self
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
            .map(|layer| layer.flat_map_recursive_par_internal(f.clone()))
            .collect();

        Layer::new_from_shapes_and_layers(shapes, sublayers)
            .with_props_inheritable(self.props_inheritable.clone())
            .with_props(self.props.clone())
    }

    /// Oversamples every shape and moves each point with `f`, recursively. see [`Shape::distort`].
    #[must_use]
    pub fn distort<F: Fn(V2) -> V2>(&self, sample_settings: SampleSettings, f: F) -> Self {
        self.map_recursive(|shape| shape.distort(sample_settings, &f).into())
    }

    /// Dashes every shape, recursively. see [`Shape::dash`].
    #[must_use]
    pub fn dash(&self, dash_length: f32, gap_length: f32, sample_settings: SampleSettings) -> Self {
        self.flat_map_recursive_par(|shape| shape.dash(dash_length, gap_length, sample_settings))
    }

    /// Dashes every shape with a custom dash function, recursively. see [`Shape::dash_fn`].
    #[must_use]
    pub fn dash_fn<F>(&self, sample_settings: SampleSettings, f: F) -> Self
    where
        F: Fn(V2, bool, f32, f32) -> f32,
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
    pub fn filter_recursive<F: Fn(&Shape) -> bool>(&self, f: F) -> Self {
        self.filter_recursive_internal(&f)
    }
    fn filter_recursive_internal<F: Fn(&Shape) -> bool>(&self, f: &F) -> Self {
        Layer::new_from_shapes_and_layers(
            self.shapes
                .iter()
                .filter(|shape| f(shape))
                .cloned()
                .collect(),
            self.sublayers
                .iter()
                .map(|layer| layer.filter_recursive_internal(f))
                .collect(),
        )
        .with_props_inheritable(self.props_inheritable.clone())
        .with_props(self.props.clone())
    }

    /// Returns a [`Masked`] object all [`Shape`]s of the `Layer` and its sublayers, grouping parts of all [`Shape`]s into `inside` and `outside`.
    ///
    /// This is done using crate [`geo`]'s [`geo::algorithm::bool_ops::BooleanOps::clip`].
    /// see [`Shape::mask_geo`].
    ///
    /// There can be unexpected outputs for example with shapes that self-intersect.
    /// See also [`Layer::mask_brute_force_flattened`] for a different and more stable masking.
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
    pub fn mask_geo_inside_flattened<M: Into<Shape>>(
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
    pub fn mask_geo_outside_flattened<M: Into<Shape>>(
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
        self.map_recursive_par(|shape| shape.reduce_points(aggression_factor))
    }

    /// Returns a [`Masked`] object all [`Shape`]s of the `Layer` and its sublayers, grouping parts of all [`Shape`]s into `inside` and `outside`.
    ///
    /// This is done by oversampling the [`Shape`] and checking for each point if it is inside or outside the mask using [`Shape::contains_point`].
    ///
    /// see [`Shape::mask_brute_force`].
    ///
    /// see also [`Layer::mask_geo_flattened`].
    pub fn mask_brute_force_flattened<M: Into<Shape>>(
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

    fn optimize_shape_start_end(shape: &Shape) -> (V2, V2) {
        match shape {
            Shape::Path(path) => {
                let points = path.as_slice();
                if let (Some(start), Some(end)) = (points.first(), points.last()) {
                    (*start, *end)
                } else {
                    (V2::zero(), V2::zero())
                }
            }
            Shape::Circle(circle) => {
                // Approximation: the plotter enters a circle at the point nearest the pen (see `points_from`).
                let start = circle.center + V2::new(circle.radius, 0.0);
                (start, start)
            }
            Shape::Rect(rect) => {
                let start = rect.bl();
                (start, start)
            }
        }
    }

    /// Returns a new `Layer` with all of this layers [`Shape`]s (non-recursive) ordered in a way that they can be plotted with a minimum of pen travel between shapes.
    /// This is done with a greedy algorithm that always chooses the closest shape to the current position.
    ///
    /// see also [`Layer::optimize_recursive`].
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
        self.map_recursive_par(|shape| shape.translate(dist))
    }
    fn translate_mut(&mut self, dist: V2) {
        self.map_recursive_mut(|shape| shape.translate_mut(dist));
    }
}

impl Rotate for Layer {
    fn rotate(&self, angle: Angle) -> Self {
        self.map_recursive_par(|shape| shape.rotate(angle))
    }
    fn rotate_mut(&mut self, angle: Angle) {
        self.map_recursive_mut(|shape| shape.rotate_mut(angle));
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        self.map_recursive_par(|shape| shape.rotate_around(pivot, angle))
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        self.map_recursive_mut(|shape| shape.rotate_around_mut(pivot, angle));
    }
}

impl Scale for Layer {
    fn scale(&self, factor: f32) -> Self {
        self.map_recursive_par(|shape| shape.scale(factor))
    }

    fn scale_mut(&mut self, factor: f32) {
        self.map_recursive_mut(|shape| shape.scale_mut(factor));
    }
}

impl Scale2D for Layer {
    fn scale_2d(&self, factor: V2) -> Self {
        self.map_recursive_par(|shape| shape.scale_2d(factor))
    }

    fn scale_2d_mut(&mut self, factor: V2) {
        self.map_recursive_mut(|shape| shape.scale_2d_mut(factor));
    }
}

impl Fit for Layer {}

impl Mirror for Layer {
    fn mirror_x(&self) -> Self {
        self.map_recursive_par(|shape| shape.mirror_x())
    }

    fn mirror_x_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.mirror_x_mut());
    }

    fn mirror_y(&self) -> Self {
        self.map_recursive_par(|shape| shape.mirror_y())
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
        self.map_recursive_par(|shape| shape.rotate_90())
    }
    fn rotate_90_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_90_mut());
    }
    fn rotate_180(&self) -> Self {
        self.map_recursive_par(|shape| shape.rotate_180())
    }
    fn rotate_180_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_180_mut());
    }
    fn rotate_270(&self) -> Self {
        self.map_recursive_par(|shape| shape.rotate_270())
    }
    fn rotate_270_mut(&mut self) {
        self.map_recursive_mut(|shape| shape.rotate_270_mut());
    }
    fn rotate_90_around(&self, pivot: V2) -> Self {
        self.map_recursive_par(|shape| shape.rotate_90_around(pivot))
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_90_around_mut(pivot));
    }
    fn rotate_180_around(&self, pivot: V2) -> Self {
        self.map_recursive_par(|shape| shape.rotate_180_around(pivot))
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_180_around_mut(pivot));
    }
    fn rotate_270_around(&self, pivot: V2) -> Self {
        self.map_recursive_par(|shape| shape.rotate_270_around(pivot))
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        self.map_recursive_mut(|shape| shape.rotate_270_around_mut(pivot));
    }
}

impl Transform for Layer {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        self.map_recursive_par(|shape| shape.transform(matrix))
    }
    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        self.map_recursive_mut(|shape| shape.transform_mut(matrix));
    }
}
