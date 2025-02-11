use crate::{Layer, Line, Path, Shape, V2};

use geo::BooleanOps;
use geo_types::{LineString, MultiLineString, Polygon};

use itertools::Itertools;

pub struct SampleSettings {
    pub points_per_unit: f32,
}

impl SampleSettings {
    pub fn new(points_per_unit: f32) -> Self {
        Self { points_per_unit }
    }
    pub fn get_num_points_for_length(&self, length: f32) -> i32 {
        (length.abs() * self.points_per_unit).ceil() as i32
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

pub trait Plottable: Clone {
    fn get_points(&self, _: &SampleSettings) -> Vec<V2>;

    fn get_line_segments(&self, sample_settings: &SampleSettings) -> Vec<Line> {
        self.get_points(sample_settings)
            .iter()
            .tuple_windows()
            .map(|(from, to)| Line::new(*from, *to))
            .collect()
    }

    fn length(&self) -> f32;

    fn is_closed(&self) -> bool;

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

    fn get_points_and_dist_oversampled(&self, sample_settings: &SampleSettings) -> Vec<(V2, f32)> {
        let points = self.get_points(sample_settings);
        let mut dist_along_path = 0.0;
        if points.is_empty() {
            return vec![];
        }
        let mut points_oversampled = vec![(points[0], 0.0)];
        for (from, to) in points.iter().tuple_windows() {
            let num_steps = sample_settings.get_num_points_for_length(from.dist(to));
            let segment_length = from.dist(to);

            if num_steps <= 1 {
                points_oversampled.push((*to, dist_along_path + segment_length));
            } else {
                let direction = to - from;
                let new_points = (1..num_steps + 1).map(|i| {
                    let fraction = i as f32 / num_steps as f32;
                    let point = from + direction * fraction;
                    let dist = dist_along_path + segment_length * fraction;
                    (point, dist)
                });
                points_oversampled.extend(new_points);
            }
            dist_along_path += segment_length;
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
            .map(V2::as_geo_coord)
            .collect_vec();
        LineString(coords)
    }

    fn as_geo_multi_line_string(&self, sample_settings: &SampleSettings) -> MultiLineString<f32> {
        MultiLineString(vec![self.as_geo_line_string(sample_settings)])
    }

    fn get_masked(&self, mask: Shape, sample_settings: &SampleSettings) -> Masked {
        let shape_geo = self.as_geo_multi_line_string(sample_settings);
        let mask_geo = mask.as_geo_polygon(sample_settings);

        let masked_inside_geo = mask_geo.clip(&shape_geo, false);
        let masked_outside_geo = mask_geo.clip(&shape_geo, true);

        let layer_inside = Layer::from_iter(
            masked_inside_geo
                .iter()
                .map(Path::new_shape_from_geo_line_string),
        );
        let layer_outside = Layer::from_iter(
            masked_outside_geo
                .iter()
                .map(Path::new_shape_from_geo_line_string),
        );

        Masked {
            inside: layer_inside,
            outside: layer_outside,
        }
    }
}
