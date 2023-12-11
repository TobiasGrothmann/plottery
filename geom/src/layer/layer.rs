use std::slice::Iter;

use itertools::Itertools;

use crate::Shape;

pub struct Layer {
    pub shapes: Vec<Box<dyn Shape>>,
}

impl Layer {
    pub fn new() -> Self {
        Self { shapes: Vec::new() }
    }

    pub fn push<S: Shape + 'static>(&mut self, shape: S) {
        self.shapes.push(Box::new(shape));
    }

    pub fn push_boxed(&mut self, shape: Box<dyn Shape>) {
        self.shapes.push(shape);
    }

    pub fn iter(&self) -> Iter<'_, Box<dyn Shape>> {
        self.shapes.iter()
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
        }
    }
}
