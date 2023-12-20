use std::{iter::FromIterator, path::PathBuf, slice::Iter};

use itertools::Itertools;
use svg::{
    node::element::{path::Data, Path},
    Document,
};

use crate::{traits::shape::Shape, SampleSettings, V2};

pub struct Layer {
    pub shapes: Vec<Box<dyn Shape>>,
    pub sublayers: Vec<Layer>,
}

impl Layer {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            sublayers: Vec::new(),
        }
    }
    pub fn new_from(shapes: Vec<Box<dyn Shape>>) -> Self {
        Self {
            shapes,
            sublayers: Vec::new(),
        }
    }

    pub fn push<S: Shape + 'static>(&mut self, shape: S) {
        self.shapes.push(Box::new(shape));
    }
    pub fn push_boxed(&mut self, shape: Box<dyn Shape>) {
        self.shapes.push(shape);
    }

    pub fn push_layer(&mut self, layer: Layer) {
        self.sublayers.push(layer);
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn Shape>> {
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

    fn as_svg(&self, sample_settings: &SampleSettings, scale: f32) -> Document {
        let shapes_points: Vec<Vec<V2>> = self
            .iter_flattened()
            .map(|shape| shape.get_points_oversampled(sample_settings))
            .map(|points| points.iter().map(|point| point * scale).collect_vec())
            .collect();

        let bounding_box = self.bounding_box();
        let mut document = Document::new().set(
            "viewbox",
            (
                bounding_box.0.x * scale,
                bounding_box.0.y * scale,
                bounding_box.1.x * scale,
                bounding_box.1.y * scale,
            ),
        );

        for shape_points in shapes_points {
            if shape_points.len() <= 1 {
                continue;
            }

            let mut data = Data::new();
            data = data.move_to(shape_points[0].as_tuple());
            data = shape_points
                .iter()
                .skip(1)
                .fold(data, |acc, point| acc.line_to(point.as_tuple()));

            if shape_points.first().unwrap() == shape_points.last().unwrap() {
                data = data.close();
            }

            let path = Path::new()
                .set("fill", "none")
                .set("stroke", "black")
                .set("stroke-width", 1)
                .set("d", data);

            document = document.add(path);
        }
        document
    }

    pub fn write_svg(&self, sample_settings: &SampleSettings, path: PathBuf, scale: f32) {
        let document = self.as_svg(sample_settings, scale);
        svg::save(path.to_str().unwrap(), &document).unwrap();
    }

    pub fn bounding_box(&self) -> (V2, V2) {
        let mut min = V2::new(0.0, 0.0);
        let mut max = V2::new(0.0, 0.0);
        for shape in self.shapes.iter() {
            let (shape_min, shape_max) = shape.bounding_box();
            min = min.min(&shape_min);
            max = max.max(&shape_max);
        }
        (min, max)
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LayerFlattenedIterator<'a> {
    stack: Vec<&'a Layer>,
    current_layer_iterator: Option<std::slice::Iter<'a, Box<dyn Shape>>>,
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
    type Item = &'a dyn Shape;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(current_layer_iter) = &mut self.current_layer_iterator {
                if let Some(shape) = current_layer_iter.next() {
                    return Some(shape.as_ref());
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

impl Clone for Layer {
    fn clone(&self) -> Self {
        Self {
            shapes: self
                .shapes
                .iter()
                .map(|shape_box| shape_box.clone_box())
                .collect_vec(),
            sublayers: self.sublayers.clone(),
        }
    }
}

impl IntoIterator for Layer {
    type Item = Box<dyn Shape>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.shapes.into_iter()
    }
}

impl FromIterator<Box<dyn Shape>> for Layer {
    fn from_iter<I: IntoIterator<Item = Box<dyn Shape>>>(iter: I) -> Self {
        Layer {
            shapes: iter.into_iter().collect(),
            sublayers: Vec::new(),
        }
    }
}
