use itertools::Itertools;
use plottery_lib::{LARGE_EPSILON, V2};

use super::maths::get_corner_sharpness;

#[derive(Debug, Clone, Copy)]
pub struct V2Speed {
    pub point: V2,
    pub speed: f32,
}

impl V2Speed {
    pub fn compute_speed(before: V2, point: V2, after: V2, edge_slow_down_power: f32) -> f32 {
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
                let segment_length = a.dist(b);
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

        std::iter::once(0.0).chain(speeds).collect()
    }

    fn edge_pass(points: &[V2], edge_slow_down_power: f32) -> Vec<f32> {
        let mut speeds = Vec::with_capacity(points.len());
        speeds.push(0.0);

        for (before, point, after) in points.iter().tuple_windows::<(_, _, _)>() {
            speeds.push(V2Speed::compute_speed(
                *before,
                *point,
                *after,
                edge_slow_down_power,
            ));
        }

        speeds.push(0.0);
        speeds
    }

    fn get_points_with_inbetweens(points: &[V2], speeds: &[f32], accell_dist: f32) -> Vec<V2Speed> {
        let mut speed_points = Vec::with_capacity(points.len() * 3);

        if points.is_empty() {
            return speed_points;
        }

        if accell_dist <= LARGE_EPSILON {
            return points
                .iter()
                .zip(speeds.iter())
                .map(|(point, speed)| V2Speed {
                    point: *point,
                    speed: *speed,
                })
                .collect();
        }

        for i in 0..points.len() - 1 {
            let a = points[i];
            let b = points[i + 1];
            let speed_a = speeds[i].clamp(0.0, 1.0);
            let speed_b = speeds[i + 1].clamp(0.0, 1.0);

            speed_points.push(V2Speed {
                point: a,
                speed: speed_a,
            });

            let segment_length = a.dist(b);
            if segment_length <= LARGE_EPSILON {
                continue;
            }

            let a_dist_to_max_speed = (1.0 - speed_a) * accell_dist;
            let b_dist_to_max_speed = (1.0 - speed_b) * accell_dist;

            if segment_length + LARGE_EPSILON >= a_dist_to_max_speed + b_dist_to_max_speed {
                // We can reach full speed somewhere in this segment.
                let first_max_dist = a_dist_to_max_speed;
                let second_max_dist = segment_length - b_dist_to_max_speed;

                if first_max_dist > LARGE_EPSILON && first_max_dist < segment_length - LARGE_EPSILON
                {
                    speed_points.push(V2Speed {
                        point: a.lerp(b, first_max_dist / segment_length),
                        speed: 1.0,
                    });
                }

                if second_max_dist > first_max_dist + LARGE_EPSILON
                    && second_max_dist < segment_length - LARGE_EPSILON
                {
                    speed_points.push(V2Speed {
                        point: a.lerp(b, second_max_dist / segment_length),
                        speed: 1.0,
                    });
                }
                continue;
            }

            // We cannot reach full speed. The best profile is triangular with one peak.
            let dist_to_peak = 0.5 * (segment_length + accell_dist * (speed_b - speed_a));
            if dist_to_peak <= LARGE_EPSILON || dist_to_peak >= segment_length - LARGE_EPSILON {
                continue;
            }

            let peak_speed = (speed_a + dist_to_peak / accell_dist)
                .min(speed_b + (segment_length - dist_to_peak) / accell_dist)
                .min(1.0);

            if peak_speed > speed_a.max(speed_b) + LARGE_EPSILON {
                speed_points.push(V2Speed {
                    point: a.lerp(b, dist_to_peak / segment_length),
                    speed: peak_speed,
                });
            }
        }

        speed_points.push(V2Speed {
            point: *points.last().unwrap(),
            speed: speeds.last().copied().unwrap_or(0.0),
        });

        speed_points
    }
}
