use crate::{
    Angle, Layer, Line, LineIntersection, Path, PointLineRelation, Shape, LARGE_EPSILON, V2,
};

use geo::BooleanOps;
use geo_types::{LineString, MultiLineString, Polygon};

use geometry_predicates::orient2d;
use itertools::Itertools;
use rand::seq::index::sample;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug)]
pub struct Masked {
    pub inside: Layer,
    pub outside: Layer,
}

pub trait Plottable: Clone {
    fn get_points(&self, _: &SampleSettings) -> Vec<V2>;
    fn get_points_from(
        &self,
        current_drawing_head_pos: &V2,
        sample_settings: &SampleSettings,
    ) -> Vec<V2>;

    fn get_line_segments(&self, sample_settings: &SampleSettings) -> Vec<Line> {
        self.get_points(sample_settings)
            .iter()
            .tuple_windows()
            .map(|(from, to)| Line::new(*from, *to))
            .collect()
    }

    fn length(&self) -> f32;

    fn is_closed(&self) -> bool;

    fn contains_point(&self, point: &V2) -> bool; // assuming shape is closed

    fn simplify(&self, aggression_factor: f32) -> Self;

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

    fn mask_geo(&self, mask: &Shape, sample_settings: &SampleSettings) -> Masked {
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

    fn mask_geo_inside(&self, mask: &Shape, sample_settings: &SampleSettings) -> Layer {
        let shape_geo = self.as_geo_multi_line_string(sample_settings);
        let mask_geo = mask.as_geo_polygon(sample_settings);

        mask_geo
            .clip(&shape_geo, false)
            .iter()
            .map(Path::new_shape_from_geo_line_string)
            .collect()
    }

    fn mask_geo_outside(&self, mask: &Shape, sample_settings: &SampleSettings) -> Layer {
        let shape_geo = self.as_geo_multi_line_string(sample_settings);
        let mask_geo = mask.as_geo_polygon(sample_settings);

        mask_geo
            .clip(&shape_geo, true)
            .iter()
            .map(Path::new_shape_from_geo_line_string)
            .collect()
    }

    fn mask_brute_force(&self, mask: &Shape, sample_settings: &SampleSettings) -> Masked {
        let points_shape = self.get_points_oversampled(sample_settings);

        let mut inside = Layer::new();
        let mut outside = Layer::new();

        let mut current_part = Path::new_from(vec![*points_shape.first().unwrap()]);
        let mut is_inside = mask.contains_point(&current_part.get_start().unwrap());

        for point in points_shape {
            let new_inside = mask.contains_point(&point);
            if is_inside != new_inside {
                if is_inside {
                    inside.push_path(current_part);
                } else {
                    outside.push_path(current_part);
                }
                current_part = Path::new_from(vec![point]);
            }
            is_inside = new_inside;
            current_part.push(point);
        }

        if is_inside {
            inside.push_path(current_part);
        } else {
            outside.push_path(current_part);
        }

        Masked { inside, outside }
    }

    fn mask_by_intersections(&self, mask: &Shape, sample_settings: &SampleSettings) -> Masked {
        let segments_mask = mask.get_line_segments(sample_settings);

        // perturb the path to avoid points on the mask lines
        let mut perturbed_path = Path::new();
        for point in self.get_points(&sample_settings) {
            let mut perturbed_point = point;
            loop {
                let (is_on_mask_line, line_angle) =
                    is_point_on_mask_line(&perturbed_point, &segments_mask, 0.1);
                if is_on_mask_line {
                    perturbed_point =
                        perturbed_point + V2::polar(line_angle + Angle::quarter_rotation(), 0.05);
                    continue;
                }

                break;
            }
            perturbed_path.push(perturbed_point);
        }

        let segments_shape: Vec<_> = perturbed_path.get_line_segments(&sample_settings);
        let mut inside = Layer::new();
        let mut outside = Layer::new();
        let mut current_part = Path::new_from(vec![segments_shape.first().unwrap().from]);
        let mut currently_inside = mask.contains_point(&current_part.get_start().unwrap());

        // switch inside to outside on each intersection
        for segment in segments_shape {
            let intersections_sorted = get_intersections_sorted(&segment, &segments_mask);

            for intersection in intersections_sorted {
                current_part.push(intersection);
                if currently_inside {
                    inside.push_path(current_part);
                } else {
                    outside.push_path(current_part);
                }
                current_part = Path::new_from(vec![intersection]);
                currently_inside = !currently_inside;
            }

            current_part.push(segment.to);
        }

        if currently_inside {
            inside.push_path(current_part);
        } else {
            outside.push_path(current_part);
        }

        Masked { inside, outside }
    }
}

fn get_intersections_sorted(segment: &Line, segments_mask: &Vec<Line>) -> Vec<V2> {
    segments_mask
        .into_iter()
        .map(|segment_mask| segment.intersection(&segment_mask))
        .filter_map(|intersection| match intersection {
            LineIntersection::Intersection(point) => Some(point),
            _ => None,
        })
        .sorted_by_cached_key(|point| point.dist_squared(&segment.from).to_bits())
        .collect()
}

fn is_point_on_mask_line(point: &V2, segments_mask: &Vec<Line>, epsilon: f64) -> (bool, Angle) {
    for segment_mask in segments_mask {
        let orientation = orient2d(
            [segment_mask.from.x as f64, segment_mask.from.y as f64],
            [segment_mask.to.x as f64, segment_mask.to.y as f64],
            [point.x as f64, point.y as f64],
        );
        if orientation < epsilon && orientation > -epsilon {
            return (true, segment_mask.angle());
        }
    }
    (false, Angle::zero())
}
