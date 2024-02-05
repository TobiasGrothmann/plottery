use geo_types::{LineString, Polygon};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{slice::Iter, slice::IterMut};

use crate::{
    traits::{Offset, Scale, Scale2D},
    Angle, Plottable, Rotate, Rotate90, SampleSettings, Shape, V2,
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
    pub fn reverse_inplace(&mut self) {
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
    fn rotate_inplace(&mut self, angle: &Angle) {
        for point in self.iter_mut() {
            point.rotate_inplace(angle);
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
    fn rotate_around_inplace(&mut self, pivot: &V2, angle: &Angle) {
        for point in self.iter_mut() {
            point.rotate_around_inplace(pivot, angle);
        }
    }
}

impl Rotate90 for Path {
    fn rotate_90(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_90()).collect(),
        }
    }
    fn rotate_90_inplace(&mut self) {
        for point in self.iter_mut() {
            point.rotate_90_inplace();
        }
    }

    fn rotate_180(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_180()).collect(),
        }
    }
    fn rotate_180_inplace(&mut self) {
        for point in self.iter_mut() {
            point.rotate_180_inplace();
        }
    }

    fn rotate_270(&self) -> Self {
        Path {
            points: self.iter().map(|point| point.rotate_270()).collect(),
        }
    }
    fn rotate_270_inplace(&mut self) {
        for point in self.iter_mut() {
            point.rotate_270_inplace();
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
    fn rotate_90_around_inplace(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_90_around_inplace(pivot);
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
    fn rotate_180_around_inplace(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_180_around_inplace(pivot);
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
    fn rotate_270_around_inplace(&mut self, pivot: &V2) {
        for point in self.iter_mut() {
            point.rotate_270_around_inplace(pivot);
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

impl Offset for Path {
    fn offset(&self, offset: &V2) -> Self {
        Path {
            points: self.iter().map(|point| point + *offset).collect(),
        }
    }
    fn offset_inplace(&mut self, offset: &V2) {
        for point in self.iter_mut() {
            *point += *offset;
        }
    }
}

impl Scale for Path {
    fn scale(&self, scale: f32) -> Self {
        Path {
            points: self.iter().map(|point| point * scale).collect(),
        }
    }
    fn scale_inplace(&mut self, scale: f32) {
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
    fn scale_2d_inplace(&mut self, factor: &V2) {
        for point in self.iter_mut() {
            *point *= factor;
        }
    }
}
