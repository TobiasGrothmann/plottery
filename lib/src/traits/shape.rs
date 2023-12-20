use crate::{Layer, Path, V2};

use geo::BooleanOps;
use geo_types::{LineString, MultiLineString, Polygon};

use itertools::Itertools;

pub struct SampleSettings {
    pub points_per_unit: f32,
}

impl SampleSettings {
    pub fn get_num_points_for_length(&self, length: f32) -> i32 {
        (length * self.points_per_unit).ceil() as i32
    }
}

impl Default for SampleSettings {
    fn default() -> Self {
        Self {
            points_per_unit: 50.0,
        }
    }
}

pub struct Masked {
    pub inside: Layer,
    pub outside: Layer,
}

pub trait Shape {
    fn clone_box(&self) -> Box<dyn Shape>;

    fn get_points(&self, _: &SampleSettings) -> Vec<V2>;

    fn length(&self) -> f32;

    fn is_closed(&self) -> bool;

    fn bounding_box(&self) -> (V2, V2) {
        let points = self.get_points(&SampleSettings::default());
        let min = points.iter().fold(V2::new(0.0, 0.0), |acc, v| acc.min(v));
        let max = points.iter().fold(V2::new(0.0, 0.0), |acc, v| acc.max(v));
        (min, max)
    }

    fn get_points_oversampled(&self, sample_settings: &SampleSettings) -> Vec<V2> {
        let points = self.get_points(sample_settings);
        if points.is_empty() {
            return points;
        }
        let mut points_oversampled = vec![points[0]];
        for (from, to) in self.get_points(sample_settings).iter().tuple_windows() {
            let num_steps = sample_settings.get_num_points_for_length(from.dist(to));
            if num_steps <= 1 {
                points_oversampled.push(*to);
            } else {
                let direction = to - from;
                let new_points =
                    (1..num_steps + 1).map(|i| from + direction * (i as f32 / num_steps as f32));
                points_oversampled.extend(new_points);
            }
        }
        points_oversampled
    }

    fn as_geo_polygon(&self, sample_settings: &SampleSettings) -> Polygon<f32> {
        Polygon::new(
            self.as_geo_line_string(sample_settings),
            vec![LineString(Vec::new())],
        )
    }

    fn as_geo_line_string(&self, sample_settings: &SampleSettings) -> LineString<f32> {
        let coords = self
            .get_points(sample_settings)
            .iter()
            .map(|v| v.as_geo_coord())
            .collect_vec();
        LineString(coords)
    }

    fn as_geo_multi_line_string(&self, sample_settings: &SampleSettings) -> MultiLineString<f32> {
        MultiLineString(vec![self.as_geo_line_string(sample_settings)])
    }

    fn get_masked(&self, mask: Box<dyn Shape>, sample_settings: &SampleSettings) -> Masked {
        let shape_geo = self.as_geo_multi_line_string(sample_settings);
        let mask_geo = mask.as_geo_polygon(sample_settings);

        let masked_inside_geo = mask_geo.clip(&shape_geo, false);
        let masked_outside_geo = mask_geo.clip(&shape_geo, true);

        let layer_inside = Layer::from_iter(
            masked_inside_geo
                .iter()
                .map(Path::new_from_geo_line_string)
                .map(|path| Box::new(path) as Box<dyn Shape>),
        );
        let layer_outside = Layer::from_iter(
            masked_outside_geo
                .iter()
                .map(Path::new_from_geo_line_string)
                .map(|path| Box::new(path) as Box<dyn Shape>),
        );

        Masked {
            inside: layer_inside,
            outside: layer_outside,
        }
    }
}
