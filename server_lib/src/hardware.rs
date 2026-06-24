use plottery_lib::{geometry::v2i::V2i, *};

use crate::{
    accelleration::{accelleration_path::V2Speed, speed_delay_handler::SpeedDelayHandler},
    Axis,
};

pub trait HardwareExecutor {
    fn set_dir(&mut self, axis: Axis, forward: bool);
    fn step(&mut self, axis: Axis);
    fn wait_axis(&mut self, axis: Axis, delay_nanos: u32);
    fn set_enabled(&mut self, enabled: bool);
}

#[derive(Debug)]
pub struct Hardware<E: HardwareExecutor> {
    enabled: bool,

    x: i32,
    y: i32,
    head_down: bool,

    hardware_profile: crate::pins::HardwareProfile,
    executor: E,
}

impl<E: HardwareExecutor> Hardware<E> {
    pub fn new(hardware_profile: crate::pins::HardwareProfile, executor: E) -> Self {
        Self {
            enabled: false,
            x: 0,
            y: 0,
            head_down: false,
            hardware_profile,
            executor,
        }
    }

    pub fn executor(&self) -> &E {
        &self.executor
    }

    pub fn executor_mut(&mut self) -> &mut E {
        &mut self.executor
    }

    pub fn get_hardware_profile(&self) -> &crate::pins::HardwareProfile {
        &self.hardware_profile
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_head_down(&self) -> bool {
        self.head_down
    }

    pub fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * self.hardware_profile.dist_per_step_axis_cm,
            self.y as f32 * self.hardware_profile.dist_per_step_axis_cm,
        )
    }

    pub fn set_origin(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    // speed_fraction: fraction from 0 to 1 that is mapped to speed_min to speed_max
    fn move_steps(
        &mut self,
        movement: V2i,
        speed_handler: &SpeedDelayHandler,
        speed_fraction_start: f32,
        speed_fraction_end: f32,
    ) {
        self.executor.set_dir(Axis::X, movement.x > 0);
        self.executor.set_dir(Axis::Y, movement.y > 0);

        let directions_signs = V2i::new(
            if movement.x > 0 { 1 } else { -1 },
            if movement.y > 0 { 1 } else { -1 },
        );

        let mut stepped_x = 0;
        let mut stepped_y = 0;

        let movement_abs = movement.abs();
        let total_steps = movement_abs.x + movement_abs.y;
        if total_steps == 0 {
            return;
        }

        while stepped_x < movement_abs.x || stepped_y < movement_abs.y {
            let moved_fraction = (stepped_x as f32 + stepped_y as f32) / total_steps as f32;
            let speed_fraction =
                speed_fraction_start + (speed_fraction_end - speed_fraction_start) * moved_fraction;

            let delay_nanos = speed_handler
                .get_delay_nanos(speed_fraction.clamp(0.0, 1.0))
                .round() as u32;

            if Line::new(V2::new(0.0, 0.0), movement_abs.abs().to_float())
                .point_relation(V2::new(stepped_x as f32, stepped_y as f32))
                == PointLineRelation::Left
            {
                self.executor.wait_axis(Axis::X, delay_nanos);
                stepped_x += 1;
                self.x += directions_signs.x;
                self.executor.step(Axis::X);
            } else {
                self.executor.wait_axis(Axis::Y, delay_nanos);
                stepped_y += 1;
                self.y += directions_signs.y;
                self.executor.step(Axis::Y);
            }
        }
    }

    pub fn move_to(
        &mut self,
        speed_fraction_start: f32,
        pos: V2Speed,
        speed_handler: &SpeedDelayHandler,
    ) {
        let delta = ((pos.point - self.get_pos()) / self.hardware_profile.dist_per_step_axis_cm)
            .round_to_int();
        if delta.is_zero() {
            return;
        }
        self.move_steps(delta, speed_handler, speed_fraction_start, pos.speed);
    }

    // TODO: avoid mistakes with changing pen pressures
    pub fn set_head(
        &mut self,
        down: bool,
        head_tracel_beyond_paper_cm: f32,
        accelleration_dist: f32,
        speed_handler: SpeedDelayHandler,
    ) {
        if self.head_down == down {
            return;
        }

        self.executor.set_dir(Axis::Head, down);

        let head_travel_cm =
            self.hardware_profile.head_travel_to_touch_cm + head_tracel_beyond_paper_cm;
        let head_travel_steps = self.hardware_profile.steps_for_cm_head(head_travel_cm);

        for i in 0..head_travel_steps {
            let fraction = i as f32 / head_travel_steps as f32;
            let current_head_travel_cm = head_travel_cm * fraction;
            let speed_fraction_acc: f32 = current_head_travel_cm / accelleration_dist;
            let speed_fraction_decc: f32 =
                (head_travel_cm - current_head_travel_cm) / accelleration_dist;
            let speed_fraction = speed_fraction_acc.min(speed_fraction_decc).clamp(0.0, 1.0);

            let delay_nanos = speed_handler.get_delay_nanos(speed_fraction.clamp(0.0, 1.0)) as u32;
            self.executor.wait_axis(Axis::Head, delay_nanos);
            self.executor.step(Axis::Head);
        }

        self.head_down = down;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }

        self.executor.set_enabled(enabled);
        self.enabled = enabled;
    }

    pub fn play_freq(&mut self, axis: &Axis, frequency: f32, duration_s: f32) {
        const SEC_NANOS: f64 = 1_000_000_000.0;

        let period_nanos = (SEC_NANOS / frequency as f64).round() as u64;
        let half_period_nanos = period_nanos / 2;

        let mut steps = ((SEC_NANOS * duration_s as f64) / half_period_nanos as f64).round() as u64;
        steps += steps % 2;

        let mut forward = true;
        for _ in 0..steps {
            self.executor.set_dir(*axis, forward);
            forward = !forward;

            self.executor.wait_axis(*axis, half_period_nanos as u32);
            self.executor.step(*axis);
        }
    }
}

impl<E: HardwareExecutor> Drop for Hardware<E> {
    fn drop(&mut self) {
        self.set_enabled(false);
    }
}
