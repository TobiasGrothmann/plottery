use std::iter::once;

use itertools::Itertools;
use plottery_lib::{LARGE_EPSILON, V2};

use super::maths::get_corner_sharpness;

#[derive(Debug)]
pub struct V2Speed {
    pub point: V2,
    pub speed: f32,
}

impl V2Speed {
    pub fn compute_speed(before: &V2, point: &V2, after: &V2, edge_slow_down_power: f32) -> f32 {
        let sharpness = get_corner_sharpness(before, point, after);
        1.0 - sharpness.powf(edge_slow_down_power)
    }
}

#[derive(Debug)]
pub struct AccellerationPath {
    pub points: Vec<V2Speed>,
}

impl AccellerationPath {
    pub fn new(points: &[V2], accell_dist: f32, corner_slowdown_power: f32) -> Self {
        let points_dedup: Vec<V2> = points.iter().dedup().cloned().collect();
        if points_dedup.len() < 2 {
            return Self {
                points: points_dedup
                    .iter()
                    .map(|point| V2Speed {
                        point: *point,
                        speed: 0.0,
                    })
                    .collect::<Vec<_>>(),
            };
        }

        let corner_sharpness_speeds = Self::edge_pass(&points_dedup, corner_slowdown_power);

        let accelleration_speeds = Self::accelleration_pass(
            points_dedup.iter().cloned(),
            corner_sharpness_speeds.iter().cloned(),
            accell_dist,
        );
        let decelleration_speeds = Self::accelleration_pass(
            points_dedup.iter().rev().cloned(),
            corner_sharpness_speeds.iter().rev().cloned(),
            accell_dist,
        );

        let min_speeds: Vec<f32> = accelleration_speeds
            .iter()
            // decelleration speeds are iterated in reverse
            .zip(decelleration_speeds.iter().rev())
            .map(|(acc, dec)| acc.min(*dec))
            .collect();

        let points_with_inbetweens =
            Self::get_points_with_inbetweens(&points_dedup, &min_speeds, accell_dist);

        Self {
            points: points_with_inbetweens,
        }
    }

    fn accelleration_pass(
        points: impl Iterator<Item = V2>,
        corner_sharpness_speeds: impl Iterator<Item = f32>,
        accell_dist: f32,
    ) -> Vec<f32> {
        let mut speed = 0.0;
        let speeds = points.zip(corner_sharpness_speeds).tuple_windows().map(
            |((a, _a_speed), (b, b_speed))| {
                let segment_length = a.dist(&b);
                let max_acc = segment_length / accell_dist;

                if speed < b_speed {
                    // we need to accellerate
                    if speed + max_acc >= b_speed {
                        // we can reach max speed
                        speed = b_speed;
                    } else {
                        // we can't reach max speed
                        speed += max_acc;
                    }
                    return speed;
                } else {
                    // set speed to next speed, decelleration is handled in reverse pass
                    speed = b_speed;
                }
                speed
            },
        );

        once(0.0).chain(speeds).collect()
    }

    fn edge_pass(points: &[V2], edge_slow_down_power: f32) -> Vec<f32> {
        let speeds = points
            .iter()
            .tuple_windows::<(_, _, _)>()
            .map(|(before, point, after)| {
                V2Speed::compute_speed(before, point, after, edge_slow_down_power)
            });

        once(0.0).chain(speeds).chain(once(0.0)).collect()
    }

    fn get_points_with_inbetweens(points: &[V2], speeds: &[f32], accell_dist: f32) -> Vec<V2Speed> {
        let mut speed_points = Vec::with_capacity(points.len() * 3);

        let mut speed = 0.0;
        for ((a, _speed_a), (b, speed_b)) in points.iter().zip(speeds.iter()).tuple_windows() {
            speed_points.push(V2Speed { point: *a, speed });

            let segment_length = a.dist(b);

            let a_needed_dist_to_max_speed = (1.0 - speed) * accell_dist;
            let b_needed_dist_to_max_speed = (1.0 - *speed_b) * accell_dist;

            if speed + LARGE_EPSILON >= 1.0 && speed_b + LARGE_EPSILON >= 1.0 {
                // we are already at max speed and stay there
                speed = 1.0;
            } else if segment_length >= a_needed_dist_to_max_speed + b_needed_dist_to_max_speed {
                // we can accellerate and decellerate to 1.0 completely
                // adding two points with speed 1.0
                let a_factor_to_peak = a_needed_dist_to_max_speed / segment_length;
                let b_factor_to_peak = b_needed_dist_to_max_speed / segment_length;
                speed = 1.0;
                speed_points.push(V2Speed {
                    point: a + (b - a) * a_factor_to_peak,
                    speed,
                });
                speed_points.push(V2Speed {
                    point: b + (a - b) * b_factor_to_peak,
                    speed,
                });
                speed = *speed_b; // at the end of segment speed is at speed_b
            } else if segment_length >= a_needed_dist_to_max_speed {
                // we can accellerate to a peak in between
                // adding one point with appropriate speed
                let dist_needed_exceeding_length =
                    (a_needed_dist_to_max_speed + b_needed_dist_to_max_speed) - segment_length;
                // distribute the missing speed equally
                let a_dist_to_midpoint =
                    a_needed_dist_to_max_speed - dist_needed_exceeding_length / 2.0;
                let b_dist_to_midpoint =
                    b_needed_dist_to_max_speed - dist_needed_exceeding_length / 2.0;
                let peak_point = a + (b - a) * (a_dist_to_midpoint / segment_length);
                // speed at midpoint is min of accelleration from a and decelleration to b
                speed += (a_dist_to_midpoint / accell_dist)
                    .min(speed_b + b_dist_to_midpoint / accell_dist);
                speed_points.push(V2Speed {
                    point: peak_point,
                    speed,
                });
                speed = *speed_b; // at the end of segment speed is at speed_b
            } else {
                speed = *speed_b;
            }
        }

        speed_points.push(V2Speed {
            point: *points.last().unwrap(),
            speed: 0.0,
        });

        speed_points
    }
}
