use std::iter::once;

use crate::maths::get_corner_sharpness;
use itertools::Itertools;
use plottery_lib::V2;

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

pub struct AccellerationPath {
    pub points: Vec<V2Speed>,
}

impl AccellerationPath {
    pub fn new(points: &Vec<V2>, accell_dist: f32, edge_slow_down_power: f32) -> Self {
        let points_dedup: Vec<V2> = points.iter().dedup().cloned().collect();
        if points_dedup.len() < 2 {
            Self {
                points: points_dedup
                    .iter()
                    .map(|point| V2Speed {
                        point: *point,
                        speed: 0.0,
                    })
                    .collect(),
            };
        }

        let corner_sharpness_speeds = Self::edge_pass(&points_dedup, edge_slow_down_power);

        let accelleration_speeds = Self::accelleration_pass(
            points_dedup.iter().cloned(),
            corner_sharpness_speeds.iter().cloned(),
            accell_dist,
        );
        let mut decelleration_speeds = Self::accelleration_pass(
            points_dedup.iter().rev().cloned(),
            corner_sharpness_speeds.iter().rev().cloned(),
            accell_dist,
        );
        decelleration_speeds.reverse();

        let min_speeds: Vec<f32> = accelleration_speeds
            .iter()
            .zip(decelleration_speeds.iter().rev())
            .map(|(acc, dec)| acc.min(*dec))
            .collect();

        let points_with_inbetweens =
            Self::get_points_with_inbetweens(&points_dedup, &min_speeds, accell_dist);

        // println!("corner_sharpness_speeds: {:?}", corner_sharpness_speeds);
        // println!("min_speeds: {:?}", min_speeds);
        // for point in points_with_inbetweens.iter() {
        //     println!("point: {:?}", point);
        // }

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

    fn edge_pass(points: &Vec<V2>, edge_slow_down_power: f32) -> Vec<f32> {
        let speeds = points
            .iter()
            .tuple_windows::<(_, _, _)>()
            .map(|(before, point, after)| {
                V2Speed::compute_speed(before, point, after, edge_slow_down_power)
            });

        once(0.0).chain(speeds).chain(once(0.0)).collect()
    }

    fn get_points_with_inbetweens(
        points: &Vec<V2>,
        speeds: &Vec<f32>,
        accell_dist: f32,
    ) -> Vec<V2Speed> {
        let mut speed_points = Vec::with_capacity(points.len() * 3);

        let mut speed = 0.0;
        for ((a, _speed_a), (b, speed_b)) in points.iter().zip(speeds.iter()).tuple_windows() {
            speed_points.push(V2Speed {
                point: *a,
                speed: speed,
            });

            let segment_length = a.dist(b);
            let max_acc_change = segment_length / accell_dist;

            let a_needed_dist_to_max_speed = (1.0 - speed) * accell_dist;
            let b_needed_dist_to_max_speed = (1.0 - *speed_b) * accell_dist;

            let a_needed_dist_to_b_speed = (speed_b - speed) * accell_dist;

            if segment_length > a_needed_dist_to_max_speed + b_needed_dist_to_max_speed {
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
            } else if segment_length > a_needed_dist_to_b_speed {
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
                speed = speed
                    + (a_dist_to_midpoint / accell_dist)
                        .min(speed_b + b_dist_to_midpoint / accell_dist);
                speed_points.push(V2Speed {
                    point: peak_point,
                    speed,
                });
                speed = *speed_b // at the end of segment speed is at speed_b
            } else {
                speed += max_acc_change;
            }
        }

        speed_points.push(V2Speed {
            point: *points.last().unwrap(),
            speed: 0.0,
        });

        speed_points
    }
}
