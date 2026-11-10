#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    use plottery_lib::V2;

    use crate::{
        accelleration::{accelleration_path::V2Speed, speed_delay_handler::SpeedDelayHandler},
        estimation::TimeEstimatorExecutor,
        hardware::{Hardware, HardwareExecutor},
        pins::HARDWARE_CONSTS,
        Axis,
    };

    const SPEED_CM_PER_S: f32 = 4.0;

    /// Moves from the origin to `target` at constant speed on the estimator
    /// and returns the time it took and where the hardware thinks it is.
    fn move_from_origin(target: V2) -> (f32, V2) {
        let profile = HARDWARE_CONSTS.hardware_profile;
        let mut hardware = Hardware::new(profile, TimeEstimatorExecutor::new());
        let constant_speed = SpeedDelayHandler::new(
            SPEED_CM_PER_S,
            SPEED_CM_PER_S,
            profile.dist_per_step_axis_cm,
        );
        hardware.move_to(
            1.0,
            V2Speed {
                point: target,
                speed: 1.0,
            },
            &constant_speed,
        );
        (
            hardware.executor().duration().as_secs_f32(),
            hardware.get_pos(),
        )
    }

    fn point_at(angle_deg: f32, length_cm: f32) -> V2 {
        let (sin, cos) = angle_deg.to_radians().sin_cos();
        V2::new(length_cm * cos, length_cm * sin)
    }

    #[test]
    fn speed_is_the_pen_speed_along_the_path_at_every_angle() {
        let length_cm = 10.0;
        let expected_secs = length_cm / SPEED_CM_PER_S;
        let step_secs = HARDWARE_CONSTS.hardware_profile.dist_per_step_axis_cm / SPEED_CM_PER_S;
        for angle in [0.0, 30.0, 45.0, 60.0, 90.0, 135.0, 200.0, 315.0] {
            let (secs, _) = move_from_origin(point_at(angle, length_cm));
            assert!(
                (secs - expected_secs).abs() < 2.0 * step_secs,
                "{angle}°: took {secs} s, expected {expected_secs} s"
            );
        }
    }

    #[test]
    fn move_ends_on_the_target() {
        let step_cm = HARDWARE_CONSTS.hardware_profile.dist_per_step_axis_cm;
        let target = point_at(30.0, 10.0);
        let (_, pos) = move_from_origin(target);
        assert!(
            (pos.x - target.x).abs() <= step_cm && (pos.y - target.y).abs() <= step_cm,
            "ended at {pos:?}, target {target:?}"
        );
    }

    #[test]
    fn zero_length_move_takes_no_time() {
        let (secs, pos) = move_from_origin(V2::zero());
        assert_eq!(secs, 0.0);
        assert_eq!(pos, V2::zero());
    }

    /// Counts steps and raises the abort flag after `abort_after` of them.
    struct AbortingExecutor {
        steps: [i32; 2],
        abort_after: i32,
        flag: Arc<AtomicBool>,
    }

    impl HardwareExecutor for AbortingExecutor {
        fn set_dir(&mut self, _axis: Axis, _forward: bool) {}
        fn step(&mut self, axis: Axis) {
            match axis {
                Axis::X => self.steps[0] += 1,
                Axis::Y => self.steps[1] += 1,
                _ => {}
            }
            if self.steps[0] + self.steps[1] == self.abort_after {
                self.flag.store(true, Ordering::Relaxed);
            }
        }
        fn wait_axis(&mut self, _axis: Axis, _delay_nanos: u32) {}
        fn wait_xy(&mut self, _delay_nanos: u32) {}
        fn set_enabled(&mut self, _enabled: bool) {}
    }

    #[test]
    fn abort_stops_a_move_and_keeps_position_consistent() {
        let profile = HARDWARE_CONSTS.hardware_profile;
        let flag = Arc::new(AtomicBool::new(false));
        let executor = AbortingExecutor {
            steps: [0, 0],
            abort_after: 120,
            flag: flag.clone(),
        };
        let mut hardware = Hardware::new(profile, executor).with_abort_flag(flag);
        let speed = SpeedDelayHandler::new(1.0, 1.0, profile.dist_per_step_axis_cm);
        hardware.move_to(
            1.0,
            V2Speed {
                point: point_at(30.0, 10.0),
                speed: 1.0,
            },
            &speed,
        );

        let [x, y] = hardware.executor().steps;
        // The flag rises at step 120; the next check, every 50 steps, is at 150.
        assert_eq!(x + y, 150);
        let step_cm = profile.dist_per_step_axis_cm;
        assert_eq!(
            hardware.get_pos(),
            V2::new(x as f32 * step_cm, y as f32 * step_cm)
        );
    }
}
