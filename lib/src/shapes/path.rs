use geo_types::{LineString, Polygon};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{slice::Iter, slice::IterMut};

use crate::{
    geometry::TransformMatrix,
    traits::{Normalize, Scale, Scale2D, Transform, Translate},
    Angle, BoundingBox, Plottable, Rect, Rotate, Rotate90, SampleSettings, Shape, V2,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
    pub fn close(&mut self) {
        if !self.points.is_empty() {
            self.points.push(*self.points.first().unwrap());
        }
    }
    pub fn reverse_mut(&mut self) {
        self.points.reverse();
    }
    pub fn reverse(&self) -> Self {
        let mut points = self.points.clone();
        points.reverse();
        Self { points }
    }

    pub fn iter(&self) -> Iter<'_, V2> {
        self.points.iter()
    }
    pub fn iter_mut(&mut self) -> IterMut<'_, V2> {
        self.points.iter_mut()
    }

    pub fn get_points_ref(&self) -> &Vec<V2> {
        &self.points
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
    fn rotate_mut(&mut self, angle: &Angle) {
        for point in self.iter_mut() {
            point.rotate_mut(angle);
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
    fn rotate_around_mut(&mut self, pivot: &V2, angle: &Angle) {
        for point in self.iter_mut() {
            point.rotate_around_mut(pivot, angle);
        }
    }
}

impl Rotate90 for Path {
    fn rotate_90(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_90()).collect(),
        }
    }
    fn rotate_90_mut(&mut self) {
        for point in self.iter_mut() {
            point.rotate_90_mut();
        }
    }

    fn rotate_180(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_180()).collect(),
        }
    }
    fn rotate_180_mut(&mut self) {
        for point in self.iter_mut() {
            point.rotate_180_mut();
        }
    }

    fn rotate_270(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_270()).collect(),
        }
    }
    fn rotate_270_mut(&mut self) {
        for point in self.iter_mut() {
            point.rotate_270_mut();
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
    fn rotate_90_around_mut(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_90_around_mut(pivot);
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
    fn rotate_180_around_mut(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_180_around_mut(pivot);
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
    fn rotate_270_around_mut(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_270_around_mut(pivot);
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

impl Translate for Path {
    fn translate(&self, dist: &V2) -> Self {
        Path {
            points: self.iter().map(|point| point + *dist).collect(),
        }
    }
    fn translate_mut(&mut self, dist: &V2) {
        for point in self.iter_mut() {
            *point += *dist;
        }
    }
}

impl Scale for Path {
    fn scale(&self, scale: f32) -> Self {
        Path {
            points: self.iter().map(|point| point * scale).collect(),
        }
    }
    fn scale_mut(&mut self, scale: f32) {
        for point in self.iter_mut() {
            *point *= scale;
        }
    }
}

impl Scale2D for Path {
    fn scale_2d(&self, factor: &V2) -> Self {
        Path {
            points: self.iter().map(|point| point * factor).collect(),
        }
    }
    fn scale_2d_mut(&mut self, factor: &V2) {
        for point in self.iter_mut() {
            *point *= factor;
        }
    }
}

impl Normalize for Path {}

impl BoundingBox for Path {
    fn bounding_box(&self) -> Option<Rect> {
        let points = self.get_points(&SampleSettings::default());
        let min = points.iter().fold(None, |acc, v| match acc {
            None => Some(*v),
            Some(acc) => Some(acc.min(v)),
        });
        let max = points.iter().fold(None, |acc, v| match acc {
            None => Some(*v),
            Some(acc) => Some(acc.max(v)),
        });
        if min.is_none() || max.is_none() {
            return None;
        }
        Some(Rect::new(min.unwrap(), max.unwrap()))
    }
}

impl Transform for Path {
    fn transform(&self, matrix: &TransformMatrix) -> Self {
        Path {
            points: self.iter().map(|point| matrix.mul_vector(point)).collect(),
        }
    }
    fn transform_mut(&mut self, matrix: &TransformMatrix) {
        for point in self.iter_mut() {
            *point = matrix.mul_vector(point);
        }
    }
}
