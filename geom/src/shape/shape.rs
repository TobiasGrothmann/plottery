use crate::vec2::V2;

use itertools::Itertools;

pub struct SampleSettings {
    pub points_per_unit: f32,
}

impl SampleSettings {
    pub fn default() -> Self {
        Self {
            points_per_unit: 50.0,
        }
    }
    pub fn get_num_points_for_length(&self, length: f32) -> i32 {
        (length * self.points_per_unit as f32).ceil() as i32
    }
}

pub trait Shape {
    fn clone_box(&self) -> Box<dyn Shape>;

    fn get_points(&self, _: &SampleSettings) -> Vec<V2>;

    fn get_points_oversampled(&self, sample_settings: &SampleSettings) -> Vec<V2> {
        let points = self.get_points(sample_settings);
        if points.len() == 0 {
            return points;
        }
        let mut points_oversampled = vec![points[0].clone()];
        for (from, to) in self.get_points(sample_settings).iter().tuple_windows() {
            let num_steps = sample_settings.get_num_points_for_length(from.dist(to));
            if num_steps <= 1 {
                points_oversampled.push(to.clone());
            } else {
                let direction = to - from;
                let new_points =
                    (1..num_steps + 1).map(|i| from + &direction * (i as f32 / num_steps as f32));
                points_oversampled.extend(new_points);
            }
        }
        points_oversampled
    }
}
