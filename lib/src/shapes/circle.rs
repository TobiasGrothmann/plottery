use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    traits::{ClosestPoint, Mirror, Normalize, Scale, Translate},
    Angle, BoundingBox, Plottable, Rect, Rotate, Rotate90, SampleSettings, Shape, LARGE_EPSILON,
    V2,
};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct Circle {
    pub center: V2,
    pub radius: f32,
}

impl Circle {
    pub fn new(center: V2, radius: f32) -> Self {
        Self { center, radius }
    }
    pub fn new_shape(center: V2, radius: f32) -> Shape {
        Shape::Circle(Self { center, radius })
    }

    pub fn area(&self) -> f32 {
        self.radius.powi(2) * PI
    }
    pub fn circumference(&self) -> f32 {
        self.radius * 2.0 * PI
    }

    pub fn get_intersections(&self, other: &Circle) -> Vec<V2> {
        let delta = other.center - self.center;
        let dist = delta.len();
        let radius_sum = self.radius + other.radius;
        let radius_difference = (self.radius - other.radius).abs();

        // circles are far apart
        if dist > radius_sum {
            return vec![];
        }
        // one circle is contained in the other
        if dist < radius_difference {
            return vec![];
        }
        // circles are the same
        if dist <= LARGE_EPSILON && radius_difference <= LARGE_EPSILON {
            return vec![];
        }
        // circles are tangent
        if (dist - radius_sum).abs() <= LARGE_EPSILON {
            return vec![self.center + V2::polar(delta.angle(), self.radius)];
        }
        // circles are tangent (one inside the other)
        if (dist - radius_difference).abs() <= LARGE_EPSILON {
            if self.radius > other.radius {
                return vec![self.center + V2::polar(delta.angle(), self.radius)];
            } else {
                return vec![
                    other.center
                        + V2::polar(delta.angle() + Angle::from_rotations(0.5), other.radius),
                ];
            }
        }

        // regular two intersections
        let a = (self.radius.powi(2) - other.radius.powi(2) + dist.powi(2)) / (2.0 * dist);
        let x2 = self.center.x + delta.x * a / dist;
        let y2 = self.center.y + delta.y * a / dist;
        let h = (self.radius.powi(2) - a.powi(2)).sqrt();
        let rx = -delta.y * (h / dist);
        let ry = delta.x * (h / dist);
        vec![V2::new(x2 + rx, y2 + ry), V2::new(x2 - rx, y2 - ry)]
    }

    fn get_points(&self, start_angle: Angle, sample_settings: SampleSettings) -> Vec<V2> {
        let num_samples = sample_settings
            .get_num_points_for_length(self.circumference())
            .max(8);
        let angle_per_step = 2.0 * PI / num_samples as f32;
        (0..num_samples + 1)
            .map(|i| {
                self.center
                    + V2::polar(
                        Angle::from_rad(i as f32 * angle_per_step) + start_angle,
                        self.radius,
                    )
            })
            .collect()
    }

    /// This utility function returns a path to draw a circle with a given thickness disregarding the pen width.
    /// It creates a spiralling path from the circle's radius towards the center with a given `thickness`.
    pub fn with_thickness_towards_center(
        &self,
        mut thickness: f32,
        pen_width: f32,
        sample_settings: SampleSettings,
    ) -> super::Path {
        thickness = thickness.min(self.radius - pen_width * 0.5);
        let num_rotations = thickness / pen_width;
        let circumference = 2.0 * PI * self.radius;
        let resolution = sample_settings.get_num_points_for_length(circumference);

        let outside = (0..resolution + 1).map(|i| {
            let rot = i as f32 / resolution as f32;
            let angle = Angle::from_rotations(rot);
            self.center + V2::polar(angle, self.radius)
        });

        let mut last_angle = Angle::zero();
        let outside_and_spiral: Vec<V2> = outside
            .chain(
                (1..(resolution as f32 * num_rotations - 1.0).round() as i32).map(|i| {
                    let angle = Angle::from_rotations(i as f32 / resolution as f32);
                    let factor = i as f32 / (num_rotations * resolution as f32);
                    let radius = self.radius - thickness * factor;
                    last_angle = angle;
                    self.center + V2::polar(angle, radius)
                }),
            )
            .collect();

        let inside = (0..resolution + 1).map(|i| {
            let rot = i as f32 / resolution as f32;
            let angle = Angle::from_rotations(rot);
            self.center + V2::polar(last_angle + angle, self.radius - thickness)
        });

        outside_and_spiral.into_iter().chain(inside).collect()
    }
}

impl Plottable for Circle {
    fn get_points(&self, sample_settings: SampleSettings) -> Vec<V2> {
        self.get_points(Angle::zero(), sample_settings)
    }

    fn get_points_from(
        &self,
        current_drawing_head_pos: V2,
        sample_settings: SampleSettings,
    ) -> Vec<V2> {
        if self.center == current_drawing_head_pos {
            return self.get_points(Angle::zero(), sample_settings);
        }
        let start_angle = (current_drawing_head_pos - self.center).angle();
        self.get_points(start_angle, sample_settings)
    }

    fn length(&self) -> f32 {
        self.circumference()
    }

    fn is_closed(&self) -> bool {
        true
    }

    fn contains_point(&self, point: V2) -> bool {
        point.dist(self.center) <= self.radius
    }

    fn reduce_points(&self, _aggression_factor: f32) -> Self {
        *self
    }
}

impl Rotate for Circle {
    fn rotate(&self, angle: Angle) -> Self {
        Circle {
            center: self.center.rotate(angle),
            radius: self.radius,
        }
    }
    fn rotate_mut(&mut self, angle: Angle) {
        self.center.rotate_mut(angle);
    }

    fn rotate_around(&self, pivot: V2, angle: Angle) -> Self {
        Circle {
            center: self.center.rotate_around(pivot, angle),
            radius: self.radius,
        }
    }
    fn rotate_around_mut(&mut self, pivot: V2, angle: Angle) {
        self.center.rotate_around_mut(pivot, angle);
    }
}

impl Rotate90 for Circle {
    fn rotate_90(&self) -> Self {
        Circle {
            center: self.center.rotate_90(),
            radius: self.radius,
        }
    }
    fn rotate_90_mut(&mut self) {
        self.center.rotate_90_mut();
    }

    fn rotate_180(&self) -> Self {
        Circle {
            center: self.center.rotate_180(),
            radius: self.radius,
        }
    }
    fn rotate_180_mut(&mut self) {
        self.center.rotate_180_mut();
    }

    fn rotate_270(&self) -> Self {
        Circle {
            center: self.center.rotate_270(),
            radius: self.radius,
        }
    }
    fn rotate_270_mut(&mut self) {
        self.center.rotate_270_mut();
    }

    fn rotate_90_around(&self, pivot: V2) -> Self {
        Circle {
            center: self.center.rotate_90_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_90_around_mut(&mut self, pivot: V2) {
        self.center.rotate_90_around_mut(pivot);
    }

    fn rotate_180_around(&self, pivot: V2) -> Self {
        Circle {
            center: self.center.rotate_180_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_180_around_mut(&mut self, pivot: V2) {
        self.center.rotate_180_around_mut(pivot);
    }

    fn rotate_270_around(&self, pivot: V2) -> Self {
        Circle {
            center: self.center.rotate_270_around(pivot),
            radius: self.radius,
        }
    }
    fn rotate_270_around_mut(&mut self, pivot: V2) {
        self.center.rotate_270_around_mut(pivot);
    }
}

impl Translate for Circle {
    fn translate(&self, dist: V2) -> Self {
        Circle {
            center: self.center + dist,
            radius: self.radius,
        }
    }
    fn translate_mut(&mut self, dist: V2) {
        self.center += dist;
    }
}

impl Scale for Circle {
    fn scale(&self, scale: f32) -> Self {
        Circle {
            center: self.center * scale,
            radius: self.radius * scale,
        }
    }
    fn scale_mut(&mut self, scale: f32) {
        self.center *= scale;
        self.radius *= scale;
    }
}

impl Mirror for Circle {
    fn mirror_x(&self) -> Self {
        Circle {
            center: self.center.mirror_x(),
            radius: self.radius,
        }
    }
    fn mirror_x_mut(&mut self) {
        self.center.mirror_x_mut();
    }

    fn mirror_y(&self) -> Self {
        Circle {
            center: self.center.mirror_y(),
            radius: self.radius,
        }
    }
    fn mirror_y_mut(&mut self) {
        self.center.mirror_y_mut();
    }
}

impl Normalize for Circle {}

impl BoundingBox for Circle {
    fn bounding_box(&self) -> Option<Rect> {
        let min = self.center - V2::xy(self.radius);
        let max = self.center + V2::xy(self.radius);
        Some(Rect::new(min, max))
    }
}

impl ClosestPoint for Circle {
    fn closest_point(&self, _: SampleSettings, point: V2) -> Option<V2> {
        let direction = point - self.center;
        if direction == V2::zero() {
            // point is at the center
            return Some(self.center + V2::new(self.radius, 0.0));
        }
        Some(self.center + direction.normalize_to(self.radius))
    }
}
