use plottery_lib::{geometry::v2i::V2i, *};
use rocket::figment::value::Map;
use std::time::Duration;
use tokio::time::Instant;

use crate::{
    accelleration_path::V2Speed, pins::PIN_SETTINGS, speed_delay_handler::SpeedDelayHandler,
};

#[derive(Debug)]
pub struct Hardware {
    enabled: bool,

    x: i32,
    y: i32,
    head_down: bool,

    last_steps_timestamp: Map<Axis, Instant>,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Axis {
    X,
    Y,
    HEAD,
}

impl Hardware {
    pub fn new() -> Self {
        Hardware {
            enabled: false,
            x: 0,
            y: 0,
            head_down: false,
            last_steps_timestamp: Map::from([
                (Axis::X, Instant::now()),
                (Axis::Y, Instant::now()),
                (Axis::HEAD, Instant::now()),
            ]),
        }
    }

    pub fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * PIN_SETTINGS.dist_per_step_axis_cm,
            self.y as f32 * PIN_SETTINGS.dist_per_step_axis_cm,
        )
    }

    fn set_dir(&mut self, axis: Axis, forward: bool) {
        // TODO: set gpio direction pins (if raspi)
        println!("Setting direction for {:?} to {:?}", axis, forward);
    }

    // speed_fraction: fraction from 0 to 1 that is mapped to speed_min to speed_max
    fn step(&mut self, axis: Axis, speed_handler: &SpeedDelayHandler, speed_fraction: f32) {
        let delay = Duration::new(
            0,
            speed_handler.get_delay_nanos(speed_fraction.clamp(0.0, 1.0)),
        );
        let delay_until = self.last_steps_timestamp[&axis] + delay;

        while delay_until >= Instant::now() {
            // wait
        }
        // TODO set gpio step pins (if raspi)

        self.last_steps_timestamp.insert(axis, Instant::now());
    }

    fn move_steps(
        &mut self,
        movement: &V2i,
        speed_handler: &SpeedDelayHandler,
        speed_fraction_start: f32,
        speed_fraction_end: f32,
    ) {
        println!("Moving steps {:?}", movement);
        self.set_dir(Axis::X, movement.x > 0);
        self.set_dir(Axis::Y, movement.y > 0);

        let directions_signs = V2i::new(
            if movement.x > 0 { 1 } else { -1 },
            if movement.y > 0 { 1 } else { -1 },
        );

        let mut stepped_x = 0;
        let mut stepped_y = 0;

        let movement_abs = movement.abs();
        let total_steps = movement_abs.x + movement_abs.y;

        while stepped_x < movement_abs.x || stepped_y < movement_abs.y {
            let moved_fraction = stepped_x as f32 + stepped_y as f32 / total_steps as f32;
            let speed_fraction =
                speed_fraction_start + (speed_fraction_end - speed_fraction_start) * moved_fraction;

            if Line::new(V2::new(0.0, 0.0), movement_abs.abs().to_float())
                .point_relation(&V2::new(stepped_x as f32, stepped_y as f32))
                == PointLineRelation::Left
            {
                stepped_x += 1;
                self.x += directions_signs.x;
                self.step(Axis::X, &speed_handler, speed_fraction);
            } else {
                stepped_y += 1;
                self.y += directions_signs.y;
                self.step(Axis::Y, &speed_handler, speed_fraction);
            }
        }
    }

    pub fn move_to(
        &mut self,
        speed_fraction_start: f32,
        pos: &V2Speed,
        speed_handler: &SpeedDelayHandler,
    ) {
        let delta =
            ((pos.point - self.get_pos()) / PIN_SETTINGS.dist_per_step_axis_cm).round_to_int();
        if delta == V2i::new(0, 0) {
            return;
        }
        println!("Moving from {:?} to {:?}", self.get_pos(), pos.point);
        self.move_steps(&delta, speed_handler, speed_fraction_start, pos.speed);
    }

    pub fn set_head(
        &mut self,
        down: bool,
        head_pressure: f32,
        accelleration_dist: f32,
        speed_handler: SpeedDelayHandler,
    ) {
        if self.head_down == down {
            return;
        }

        // TODO: move head
        self.set_dir(Axis::HEAD, down);
        let head_travel_cm = PIN_SETTINGS.head_travel_to_touch_cm
            + PIN_SETTINGS.extra_head_travel_for_pressure_cm * head_pressure;
        let head_travel_steps = PIN_SETTINGS.steps_for_cm_head(head_travel_cm);

        for i in 0..head_travel_steps {
            let fraction = i as f32 / head_travel_steps as f32;
            let current_head_travel_cm = head_travel_cm * fraction;
            let speed_fraction_acc: f32 = current_head_travel_cm / accelleration_dist;
            let speed_fraction_decc: f32 =
                (head_travel_cm - current_head_travel_cm) / accelleration_dist;
            let speed_fraction = speed_fraction_acc.min(speed_fraction_decc).clamp(0.0, 1.0);

            self.step(Axis::HEAD, &speed_handler, speed_fraction);
        }

        self.head_down = down;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        // TODO: set enable pins
        println!("Setting enabled to {:?}", enabled);
        self.enabled = enabled;
    }
}
