use std::{iter::FromIterator, path::PathBuf, slice::Iter};

use anyhow::{Ok, Result};
use itertools::Itertools;
use svg::{node::element::path::Data, Document};

use crate::{traits::plottable::Plottable, Circle, Path, Rect, Shape, V2};

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

    fn as_svg(&self, scale: f32) -> Document {
        let bounding_box = self.bounding_box();
        let mut document = Document::new()
            .set(
                "viewbox",
                (
                    bounding_box.bl().x * scale,
                    bounding_box.bl().y * scale,
                    bounding_box.tr().x * scale,
                    bounding_box.tr().y * scale,
                ),
            )
            .set("width", bounding_box.size().x * scale)
            .set("height", bounding_box.size().y * scale);

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
                    data = data.move_to(points[0].as_tuple());
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
        let document = self.as_svg(scale);
        svg::save(path.to_str().unwrap(), &document)?;
        Ok(())
    }

    pub fn bounding_box(&self) -> Rect {
        let mut min = V2::new(0.0, 0.0);
        let mut max = V2::new(0.0, 0.0);
        for shape in self.shapes.iter() {
            let shape_box = shape.bounding_box();
            min = min.min(&shape_box.bl());
            max = max.max(&shape_box.tr());
        }
        Rect::new(min, max)
    }
}

impl Default for Layer {
    fn default() -> Self {
        Self::new()
    }
}

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

impl Clone for Layer {
    fn clone(&self) -> Self {
        Self {
            shapes: self.shapes.iter().cloned().collect_vec(),
            sublayers: self.sublayers.clone(),
        }
    }
}

impl IntoIterator for Layer {
    type Item = Shape;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.shapes.into_iter()
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
