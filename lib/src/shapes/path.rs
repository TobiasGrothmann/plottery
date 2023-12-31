use geo_types::{LineString, Polygon};
use itertools::Itertools;
use std::{slice::Iter, slice::IterMut};

use crate::{Angle, Plottable, Rotate, Rotate90, SampleSettings, Shape, V2};

#[derive(Debug, Clone, Default)]
pub struct Path {
    points: Vec<V2>,
}

impl Path {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
    pub fn new_from(points: Vec<V2>) -> Self {
        Self { points }
    }
    pub fn new_shape() -> Shape {
        Shape::Path(Self { points: vec![] })
    }
    pub fn new_shape_from(points: Vec<V2>) -> Shape {
        Shape::Path(Self { points })
    }
    pub fn new_shape_from_geo_polygon(geo_polygon: Polygon<f32>) -> Shape {
        Shape::Path(Self::from_iter(
            geo_polygon.exterior().into_iter().map(V2::new_from_geo),
        ))
    }
    pub fn new_shape_from_geo_line_string(geo_line_string: &LineString<f32>) -> Shape {
        Shape::Path(Self::from_iter(
            geo_line_string.into_iter().map(V2::new_from_geo),
        ))
    }

    pub fn push(&mut self, point: V2) {
        self.points.push(point);
    }

    pub fn iter(&self) -> Iter<'_, V2> {
        self.points.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, V2> {
        self.points.iter_mut()
    }
}

impl Plottable for Path {
    fn get_points(&self, _: &SampleSettings) -> Vec<V2> {
        self.points.clone()
    }

    fn length(&self) -> f32 {
        self.points
            .iter()
            .tuple_windows()
            .fold(0.0, |acc, (from, to)| acc + from.dist(to))
    }

    fn is_closed(&self) -> bool {
        !self.points.is_empty() && self.points.first() == self.points.last()
    }
}

impl FromIterator<V2> for Path {
    fn from_iter<I: IntoIterator<Item = V2>>(iter: I) -> Self {
        Path {
            points: iter.into_iter().collect(),
        }
    }
}

impl Rotate for Path {
    fn rotate(&self, angle: &Angle) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate(angle)).collect(),
        }
    }

    fn rotate_around(&self, pivot: &V2, angle: &Angle) -> Self {
        Path {
            points: self
                .iter()
                .map(|point| point.rotate_around(pivot, angle))
                .collect(),
        }
    }
}

impl Rotate90 for Path {
    fn rotate_90(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_90()).collect(),
        }
    }

    fn rotate_180(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_180()).collect(),
        }
    }

    fn rotate_270(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_270()).collect(),
        }
    }

    fn rotate_90_around(&self, pivot: &V2) -> Self {
        Path {
            points: self
                .iter()
                .map(|point| point.rotate_90_around(pivot))
                .collect(),
        }
    }

    fn rotate_180_around(&self, pivot: &V2) -> Self {
        Path {
            points: self
                .iter()
                .map(|point| point.rotate_180_around(pivot))
                .collect(),
        }
    }

    fn rotate_270_around(&self, pivot: &V2) -> Self {
        Path {
            points: self
                .iter()
                .map(|point| point.rotate_270_around(pivot))
                .collect(),
        }
    }
}

impl IntoIterator for Path {
    type Item = V2;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.points.into_iter()
    }
}
