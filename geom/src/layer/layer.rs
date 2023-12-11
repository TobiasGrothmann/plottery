use itertools::Itertools;
use std::slice::Iter;

use crate::Shape;

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
        LayerFlattenedIterator::new(&self)
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
