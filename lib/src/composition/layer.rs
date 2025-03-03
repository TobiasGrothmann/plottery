use anyhow::{Ok, Result};
use bincode::{deserialize_from, serialize};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet, fs::File, io::Write, iter::FromIterator, path::PathBuf, rc::Rc,
    slice::Iter, vec,
};
use svg::{node::element::path::Data, Document};

use crate::{
    traits::{Normalize, Scale, Scale2D, Translate},
    Angle, BoundingBox, Circle, Masked, Path, Plottable, Rect, Rotate, SampleSettings, Shape, V2,
};

use super::path_end::PathEnd;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub shapes: Vec<Shape>,
    pub sublayers: Vec<Layer>,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            sublayers: Vec::new(),
        }
    }
    pub fn new_from(shapes: Vec<Shape>) -> Self {
        Self {
            shapes,
            sublayers: Vec::new(),
        }
    }
    pub fn new_from_shapes_and_layers(shapes: Vec<Shape>, sublayers: Vec<Layer>) -> Self {
        Self { shapes, sublayers }
    }
    pub fn new_from_file(path: &PathBuf) -> Result<Layer> {
        let file = File::open(path)?;
        let decoded: Layer = deserialize_from(&file)?;
        Ok(decoded)
    }
    pub fn new_from_binary(binary_data: &Vec<u8>) -> Result<Layer> {
        Ok(deserialize_from(binary_data.as_slice())?)
    }

    pub fn write_file(&self, path: &PathBuf) -> Result<()> {
        let encoded: Vec<u8> = serialize(self)?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }
    pub fn to_binary(&self) -> Result<Vec<u8>> {
        Ok(serialize(self)?)
    }

    pub fn push(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }
    pub fn push_path(&mut self, path: Path) {
        self.shapes.push(Shape::Path(path));
    }
    pub fn push_circle(&mut self, circle: Circle) {
        self.shapes.push(Shape::Circle(circle));
    }
    pub fn push_rect(&mut self, rect: Rect) {
        self.shapes.push(Shape::Rect(rect));
    }
    pub fn push_layer(&mut self, layer: Layer) {
        self.sublayers.push(layer);
    }
    pub fn push_layer_flat(&mut self, layer: Layer) {
        for shape in layer.iter_flattened() {
            self.shapes.push(shape.clone());
        }
    }

    pub fn iter(&self) -> Iter<'_, Shape> {
        self.shapes.iter()
    }
    pub fn iter_sublayers(&self) -> Iter<Layer> {
        self.sublayers.iter()
    }
    pub fn iter_flattened(&self) -> LayerFlattenedIterator {
        LayerFlattenedIterator::new(self)
    }

    pub fn len(&self) -> i32 {
        self.shapes.len() as i32
    }
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }
    pub fn len_recursive(&self) -> i32 {
        self.sublayers
            .iter()
            .fold(self.len(), |acc, sublayer| acc + sublayer.len_recursive())
    }
    pub fn len_sublayers(&self) -> i32 {
        self.sublayers.len() as i32
    }

    pub fn to_svg(&self, scale: f32) -> Document {
        let bounding_box = self.bounding_box();
        if bounding_box.is_none() {
            return Document::new();
        }
        let bounding_box = bounding_box.unwrap();
        let svg_max_coords: V2 = bounding_box.tr() * scale + V2::new(1.0, 1.0);
        let mut document = Document::new()
            .set("viewBox", (0, 0, svg_max_coords.x, svg_max_coords.y))
            .set("width", svg_max_coords.x)
            .set("height", svg_max_coords.y);

        let fill = "none";
        let stroke = "black";
        let stroke_width = 0.1;

        for shape in self.iter_flattened() {
            match shape {
                Shape::Circle(c) => {
                    let circle = svg::node::element::Circle::new()
                        .set("cx", c.center.x * scale)
                        .set("cy", c.center.y * scale)
                        .set("r", c.radius * scale)
                        .set("fill", fill)
                        .set("stroke", stroke)
                        .set("stroke-width", stroke_width);
                    document = document.add(circle);
                }
                Shape::Rect(r) => {
                    let bl = r.bl();
                    let size = r.size();
                    let rect = svg::node::element::Rectangle::new()
                        .set("x", bl.x * scale)
                        .set("y", bl.y * scale)
                        .set("width", size.x * scale)
                        .set("height", size.y * scale)
                        .set("fill", fill)
                        .set("stroke", stroke)
                        .set("stroke-width", stroke_width);
                    document = document.add(rect);
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
                    let path = svg::node::element::Path::new()
                        .set("d", data)
                        .set("fill", fill)
                        .set("stroke", stroke)
                        .set("stroke-width", stroke_width);
                    document = document.add(path);
                }
            }
        }
        document
    }

    pub fn write_svg(&self, path: PathBuf, scale: f32) -> Result<()> {
        let document = self.to_svg(scale);
        svg::save(path.to_str().unwrap(), &document)?;
        Ok(())
    }

    pub fn combine_shapes_flat(&self, max_angle_delta: Option<Angle>) -> Self {
        let mut combined_shapes = Layer::new();

        // flatten references, create mask of used shapes
        let mut paths = Vec::new();

        // move non combineable shapes
        for shape in self.iter_flattened() {
            match shape {
                Shape::Path(path) => {
                    if path.get_points_ref().len() <= 1 {
                        continue;
                    }
                    paths.push(path.clone());
                }
                Shape::Circle(_) => {
                    combined_shapes.push((*shape).clone());
                }
                Shape::Rect(_) => {
                    combined_shapes.push((*shape).clone());
                }
            }
        }

        let mut last_num_paths = paths.len();
        loop {
            paths = self.combine_shapes_flat_iterate(&paths, &max_angle_delta);
            if last_num_paths <= paths.len() || paths.len() == 1 {
                break;
            }
            last_num_paths = paths.len();
        }

        for path in paths {
            combined_shapes.push_path(path);
        }

        combined_shapes
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
                    let mut new_current_path = (*path_candidate).clone();
                    new_current_path.push_iter_ref(current_path.get_points_ref().iter().skip(1));
                    current_path = new_current_path;

                    current_path_start = start;
                    mask[j] = true;
                } else if current_path_start.is_compatible(&start.flipped(), max_angle_delta) {
                    // reverse candidate and prepend
                    // end -> start -> # -> current_start -> current_end
                    let mut new_current_path = (*path_candidate).clone();
                    new_current_path.reverse_mut();
                    new_current_path.push_iter_ref(current_path.get_points_ref().iter().skip(1));
                    current_path = new_current_path;

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

    pub fn map_recursive_mut<F: Fn(&mut Shape)>(&mut self, f: F) {
        let f = Rc::new(f);
        self.map_recursive_mut_internal(f)
    }
    fn map_recursive_mut_internal<F: Fn(&mut Shape)>(&mut self, f: Rc<F>) {
        for shape in &mut self.shapes {
            f(shape);
        }
        for sublayer in &mut self.sublayers {
            sublayer.map_recursive_mut_internal(f.clone());
        }
    }

    pub fn map_recursive<F: Fn(&Shape) -> Shape>(&self, f: F) -> Self {
        let f = Rc::new(f);
        self.map_shapes_recursive_internal(f)
    }
    fn map_shapes_recursive_internal<F: Fn(&Shape) -> Shape>(&self, f: Rc<F>) -> Self {
        Layer::new_from_shapes_and_layers(
            self.shapes.iter().map(|shape| f(shape)).collect(),
            self.sublayers
                .iter()
                .map(|sublayer| sublayer.map_shapes_recursive_internal(f.clone()))
                .collect(),
        )
    }

    pub fn filter_recursive<F>(&self, f: F) -> Self
    where
        F: Fn(&Shape) -> bool + Clone,
    {
        let f = Rc::new(f);
        self.filter_recursive_internal(f)
    }

    fn filter_recursive_internal<F>(&self, f: Rc<F>) -> Self
    where
        F: Fn(&Shape) -> bool,
    {
        let filtered_shapes: Vec<Shape> = self
            .shapes
            .iter()
            .filter(|shape| f(shape))
            .cloned()
            .collect();
        let filtered_sublayers: Vec<Layer> = self
            .sublayers
            .iter()
            .map(|layer| layer.filter_recursive_internal(f.clone()))
            .collect();
        Layer::new_from_shapes_and_layers(filtered_shapes, filtered_sublayers)
    }

    pub fn filter_recursive_mut<F>(&mut self, predicate: F)
    where
        F: Fn(&Shape) -> bool + Clone,
    {
        let predicate = Rc::new(predicate);
        self.filter_recursive_mut_internal(predicate)
    }

    fn filter_recursive_mut_internal<F>(&mut self, predicate: Rc<F>)
    where
        F: Fn(&Shape) -> bool,
    {
        self.shapes.retain(|shape| predicate(shape));
        for sublayer in &mut self.sublayers {
            sublayer.filter_recursive_mut_internal(predicate.clone());
        }
    }

    pub fn mask_flattened(&self, mask: &Shape, sample_settings: &SampleSettings) -> Masked {
        let mut inside = Layer::new();
        let mut outside = Layer::new();

        for shape in self.iter_flattened() {
            let masked = shape.mask_geo(mask, sample_settings);
            inside.push_layer_flat(masked.inside);
            outside.push_layer_flat(masked.outside);
        }

        Masked { inside, outside }
    }

    pub fn mask_geo_flattened_inside(
        &self,
        mask: &Shape,
        sample_settings: &SampleSettings,
    ) -> Layer {
        self.iter_flattened()
            .map(|shape| shape.mask_geo_inside(mask, sample_settings))
            .collect()
    }

    pub fn mask_geo_flattened_outside(
        &self,
        mask: &Shape,
        sample_settings: &SampleSettings,
    ) -> Layer {
        self.iter_flattened()
            .map(|shape| shape.mask_geo_outside(mask, sample_settings))
            .collect()
    }

    pub fn simplify(&self, aggression_factor: f32) -> Self {
        self.map_recursive(|shape| shape.simplify(aggression_factor))
    }

    pub fn mask_flattened_brute_force(
        &self,
        mask: &Shape,
        sample_settings: &SampleSettings,
    ) -> Masked {
        let mut inside = Layer::new();
        let mut outside = Layer::new();

        for shape in self.iter_flattened() {
            let masked = shape.mask_brute_force(mask, sample_settings);
            inside.push_layer_flat(masked.inside);
            outside.push_layer_flat(masked.outside);
        }

        Masked { inside, outside }
    }

    pub fn optimize(&self) -> Self {
        let sample_settings = SampleSettings::new(1.0);
        let starts_and_ends: Vec<_> = self
            .shapes
            .iter()
            .map(|shape| {
                let points = shape.get_points(&sample_settings);
                if points.len() == 0 {
                    return (V2::zero(), V2::zero());
                }
                (
                    points.first().unwrap().clone(),
                    points.last().unwrap().clone(),
                )
            })
            .collect();

        let mut unused_items_indices: BTreeSet<usize> = (0..self.shapes.len()).collect();

        let mut pos = V2::zero();
        let mut optimized = Layer::new();

        while unused_items_indices.len() > 0 {
            let mut best_distance = f32::INFINITY;
            let mut best_index = 0;
            let mut reversed = false;

            for unused_i in unused_items_indices.iter() {
                let dist_to_start = starts_and_ends[*unused_i].0.dist_squared(&pos);
                let dist_to_end = starts_and_ends[*unused_i].1.dist_squared(&pos);

                if dist_to_start < best_distance || dist_to_end < best_distance {
                    reversed = dist_to_end < dist_to_start;
                    best_distance = dist_to_start;
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

    pub fn optimize_recursive(&self) -> Self {
        let mut optimized = self.optimize();
        for sublayer in &self.sublayers {
            optimized.push_layer(sublayer.optimize_recursive());
        }
        optimized
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Vec<V2>>> for Layer {
    fn from(vecs: Vec<Vec<V2>>) -> Self {
        let shapes = vecs.into_iter().map(|points| points.into()).collect();
        Layer::new_from(shapes)
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
                for sublayer in layer.sublayers.iter() {
                    self.stack.push(sublayer);
                }
            } else {
                return None;
            }
        }
    }
}

impl FromIterator<Shape> for Layer {
    fn from_iter<I: IntoIterator<Item = Shape>>(iter: I) -> Self {
        Layer {
            shapes: iter.into_iter().collect(),
            sublayers: Vec::new(),
        }
    }
}

impl FromIterator<Layer> for Layer {
    fn from_iter<I: IntoIterator<Item = Layer>>(iter: I) -> Self {
        Layer::new_from_shapes_and_layers(Vec::new(), iter.into_iter().collect())
    }
}

impl Translate for Layer {
    fn translate(&self, dist: &V2) -> Self {
        self.map_recursive(|shape| shape.translate(dist))
    }
    fn translate_mut(&mut self, dist: &V2) {
        self.map_recursive_mut(|shape| shape.translate_mut(dist));
    }
}

impl Rotate for Layer {
    fn rotate(&self, angle: &crate::Angle) -> Self {
        self.map_recursive(|shape| shape.rotate(angle))
    }
    fn rotate_mut(&mut self, angle: &crate::Angle) {
        self.map_recursive_mut(|shape| shape.rotate_mut(angle));
    }

    fn rotate_around(&self, pivot: &V2, angle: &crate::Angle) -> Self {
        self.map_recursive(|shape| shape.rotate_around(pivot, angle))
    }
    fn rotate_around_mut(&mut self, pivot: &V2, angle: &crate::Angle) {
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
    fn scale_2d(&self, factor: &V2) -> Self {
        self.map_recursive(|shape| shape.scale_2d(factor))
    }

    fn scale_2d_mut(&mut self, factor: &V2) {
        self.map_recursive_mut(|shape| shape.scale_2d_mut(factor));
    }
}

impl Normalize for Layer {}

impl BoundingBox for Layer {
    fn bounding_box(&self) -> Option<Rect> {
        let mut min = None;
        let mut max = None;
        for shape in self.iter_flattened() {
            let shape_box = shape.bounding_box();
            if shape_box.is_none() {
                continue;
            }
            let shape_box = shape_box.unwrap();
            if min.is_none() {
                min = Some(shape_box.bl());
            } else {
                min = Some(min.unwrap().min(&shape_box.bl()));
            }
            if max.is_none() {
                max = Some(shape_box.tr());
            } else {
                max = Some(max.unwrap().max(&shape_box.tr()));
            }
        }
        if min.is_none() || max.is_none() {
            return None;
        }
        Some(Rect::new(min.unwrap(), max.unwrap()))
    }
}
