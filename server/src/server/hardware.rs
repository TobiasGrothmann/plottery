use crossterm::{cursor, terminal, ExecutableCommand, QueueableCommand};
use plottery_lib::{geometry::v2i::V2i, *};
use rocket::figment::value::Map;
use std::io::{stdout, Write};
use std::time::Duration;
use std::{thread, time};
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

    speed_draw: SpeedDelayHandler,
    speed_travel: SpeedDelayHandler,
    speed_head_down: SpeedDelayHandler,
    speed_head_up: SpeedDelayHandler,

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
            speed_draw: SpeedDelayHandler::new(0.75, 5.0, 0.1), // TODO!
            speed_travel: SpeedDelayHandler::new(4.0, 28.0, 0.1), // TODO!
            speed_head_down: SpeedDelayHandler::new(1.8, 20.0, 0.1), // TODO!
            speed_head_up: SpeedDelayHandler::new(35.0, 35.0, 0.1), // TODO!
            last_steps_timestamp: Map::from([
                (Axis::X, Instant::now()),
                (Axis::Y, Instant::now()),
                (Axis::HEAD, Instant::now()),
            ]),
        }
    }

    fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * PIN_SETTINGS.dist_per_step_axis_cm,
            self.y as f32 * PIN_SETTINGS.dist_per_step_axis_cm,
        )
    }

    fn set_dir(&mut self, axis: Axis, forward: bool) {
        // set gpio direction pins (if raspi)
        println!("Setting direction for {:?} to {:?}", axis, forward);
    }

    // TODO: add docs for speed_fraction
    fn step(
        &mut self,
        axis: Axis,
        speed_handler: &SpeedDelayHandler,
        speed_fraction: f32,
        stdout: &mut std::io::Stdout,
    ) {
        // set gpio step pins (if raspi)
        let delay = Duration::new(
            0,
            speed_handler
                .get_delay_micros(speed_fraction.clamp(0.0, 1.0))
                .floor() as u32
                * 1000_u32,
        );
        let delay_until = self.last_steps_timestamp[&axis] + delay;

        while delay_until >= Instant::now() {
            // wait
        }
        // TODO: step

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .write_all(
                format!(
                    "axis {:?} - delay: {} - speed fraction: {}",
                    axis,
                    delay.as_millis(),
                    speed_fraction
                )
                .as_bytes(),
            )
            .unwrap();
        stdout.queue(cursor::RestorePosition).unwrap();
        stdout.flush().unwrap();

        self.last_steps_timestamp.insert(axis, Instant::now());
    }

    fn move_steps(
        &mut self,
        movement: &V2i,
        speed_handler: SpeedDelayHandler,
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

        let mut stdout = stdout();
        stdout.execute(cursor::Hide).unwrap();
        stdout.queue(cursor::SavePosition).unwrap();

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
                self.step(Axis::X, &speed_handler, speed_fraction, &mut stdout);
            } else {
                stepped_y += 1;
                self.y += directions_signs.y;
                self.step(Axis::Y, &speed_handler, speed_fraction, &mut stdout);
            }
        }

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
        stdout.execute(cursor::Show).unwrap();
    }

    pub fn move_to(&mut self, speed_fraction_start: f32, pos: &V2Speed) {
        // TODO: avoid clone
        let speed_handler = if self.head_down {
            self.speed_draw.clone()
        } else {
            self.speed_travel.clone()
        };

        let delta =
            ((pos.point - self.get_pos()) / PIN_SETTINGS.dist_per_step_axis_cm).round_to_int();
        if delta == V2i::new(0, 0) {
            return;
        }
        println!("Moving from {:?} to {:?}", self.get_pos(), pos.point);
        self.move_steps(&delta, speed_handler, speed_fraction_start, pos.speed);
    }

    pub fn set_head(&mut self, down: bool, head_pressure: f32) {
        if self.head_down == down {
            return;
        }
        // TODO: avoid clone
        let speed_handler = if down {
            self.speed_head_down.clone()
        } else {
            self.speed_head_up.clone()
        };

        // TODO: move head
        self.set_dir(Axis::HEAD, down);
        let head_travel_cm = PIN_SETTINGS.head_travel_to_touch_cm
            + PIN_SETTINGS.extra_head_travel_for_pressure_cm * head_pressure;
        let head_travel_steps = PIN_SETTINGS.steps_for_cm_head(head_travel_cm);

        let mut stdout = stdout();
        stdout.execute(cursor::Hide).unwrap();
        stdout.queue(cursor::SavePosition).unwrap();

        for i in 0..head_travel_steps {
            let fraction = i as f32 / head_travel_steps as f32;
            // TODO: move to settings
            let acc_dist_fraction_head = 0.2; // fraction of total distance used to acc and decc head

            let speed_fraction_acc: f32 = fraction / acc_dist_fraction_head;
            let speed_fraction_decc: f32 = (1.0 - fraction) / acc_dist_fraction_head;
            let speed_fraction = speed_fraction_acc.min(speed_fraction_decc).clamp(0.0, 1.0);

            self.step(Axis::HEAD, &speed_handler, speed_fraction, &mut stdout);
        }

        stdout.queue(cursor::RestorePosition).unwrap();
        stdout
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))
            .unwrap();
        stdout.execute(cursor::Show).unwrap();

        self.head_down = down;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        // TODO: set enable pins
        println!("Setting enabled to {:?}", enabled);
        self.enabled = enabled;
    }
}
