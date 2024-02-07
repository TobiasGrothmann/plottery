use anyhow::{Ok, Result};
use bincode::{deserialize_from, serialize};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Write, iter::FromIterator, path::PathBuf, rc::Rc, slice::Iter};
use svg::{node::element::path::Data, Document};

use crate::{
    traits::{Normalize, Scale, Scale2D, Translate},
    BoundingBox, Circle, Path, Rect, Rotate, Shape, V2,
};

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
    pub fn new_from_binary(binary_datra: &Vec<u8>) -> Result<Layer> {
        Ok(deserialize_from(binary_datra.as_slice())?)
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
        let document = self.to_svg(scale);
        svg::save(path.to_str().unwrap(), &document)?;
        Ok(())
    }

    fn apply_func_to_shapes_recursive_inplace<F: Fn(&mut Shape)>(&mut self, f: F) {
        let f = Rc::new(f);
        self.apply_func_to_shapes_recursive_inplace_internal(f)
    }
    fn apply_func_to_shapes_recursive_inplace_internal<F: Fn(&mut Shape)>(&mut self, f: Rc<F>) {
        for shape in &mut self.shapes {
            f(shape);
        }
        for sublayer in &mut self.sublayers {
            sublayer.apply_func_to_shapes_recursive_inplace_internal(f.clone());
        }
    }

    fn apply_func_to_shapes_recursive<F: Fn(&Shape) -> Shape>(&self, f: F) -> Self {
        let f = Rc::new(f);
        self.apply_func_to_shapes_recursive_internal(f)
    }
    fn apply_func_to_shapes_recursive_internal<F: Fn(&Shape) -> Shape>(&self, f: Rc<F>) -> Self {
        Layer::new_from_shapes_and_layers(
            self.shapes.iter().map(|shape| f(shape)).collect(),
            self.sublayers
                .iter()
                .map(|sublayer| sublayer.apply_func_to_shapes_recursive_internal(f.clone()))
                .collect(),
        )
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

impl Translate for Layer {
    fn translate(&self, dist: &V2) -> Self {
        self.apply_func_to_shapes_recursive(|shape| shape.translate(dist))
    }
    fn translate_inplace(&mut self, dist: &V2) {
        self.apply_func_to_shapes_recursive_inplace(|shape| shape.translate_inplace(dist));
    }
}

impl Rotate for Layer {
    fn rotate(&self, angle: &crate::Angle) -> Self {
        self.apply_func_to_shapes_recursive(|shape| shape.rotate(angle))
    }
    fn rotate_inplace(&mut self, angle: &crate::Angle) {
        self.apply_func_to_shapes_recursive_inplace(|shape| shape.rotate_inplace(angle));
    }

    fn rotate_around(&self, pivot: &V2, angle: &crate::Angle) -> Self {
        self.apply_func_to_shapes_recursive(|shape| shape.rotate_around(pivot, angle))
    }
    fn rotate_around_inplace(&mut self, pivot: &V2, angle: &crate::Angle) {
        self.apply_func_to_shapes_recursive_inplace(|shape| {
            shape.rotate_around_inplace(pivot, angle)
        });
    }
}

impl Scale for Layer {
    fn scale(&self, factor: f32) -> Self {
        self.apply_func_to_shapes_recursive(|shape| shape.scale(factor))
    }

    fn scale_inplace(&mut self, factor: f32) {
        self.apply_func_to_shapes_recursive_inplace(|shape| shape.scale_inplace(factor));
    }
}

impl Scale2D for Layer {
    fn scale_2d(&self, factor: &V2) -> Self {
        self.apply_func_to_shapes_recursive(|shape| shape.scale_2d(factor))
    }

    fn scale_2d_inplace(&mut self, factor: &V2) {
        self.apply_func_to_shapes_recursive_inplace(|shape| shape.scale_2d_inplace(factor));
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
