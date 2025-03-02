use plottery_lib::{geometry::v2i::V2i, *};
use rocket::figment::value::Map;
#[cfg(feature = "raspi")]
use std::thread::sleep;
use std::time::Duration;
use tokio::time::Instant;

#[cfg(feature = "raspi")]
use rppal::gpio::{Gpio, OutputPin};

use crate::{
    accelleration_path::V2Speed, pins::PinSettings, speed_delay_handler::SpeedDelayHandler,
};

#[cfg(feature = "raspi")]
const PIN_DELAY_NANOS: u32 = 100;

#[derive(Debug)]
pub struct Hardware {
    enabled: bool,

    x: i32,
    y: i32,
    head_down: bool,

    last_steps_timestamp: Map<Axis, Instant>,

    #[cfg(feature = "raspi")]
    pins_dir: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    pins_step: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    pins_enable: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    _pins_micstep: Vec<Vec<OutputPin>>,

    pin_settings: PinSettings,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Axis {
    X,
    Y,
    Head,
}

impl Hardware {
    pub fn new(pin_settings: PinSettings) -> anyhow::Result<Self> {
        #[cfg(feature = "raspi")]
        {
            let gpio = Gpio::new()?;

            let mut pins_dir = Vec::with_capacity(4);
            let mut pins_step = Vec::with_capacity(4);
            let mut pins_enable = Vec::with_capacity(4);
            let mut pins_micstep = Vec::with_capacity(4);
            for i in 0..4 {
                pins_dir.push(gpio.get(pin_settings.dir_pins[i])?.into_output());
                pins_step.push(gpio.get(pin_settings.step_pins[i])?.into_output());
                pins_enable.push(gpio.get(pin_settings.enable_pins[i])?.into_output());

                let mut micstep = Vec::with_capacity(3);
                for j in 0..3 {
                    micstep.push(gpio.get(pin_settings.micstep_pins[j][i])?.into_output());
                }
                pins_micstep.push(micstep);
            }

            // set all low
            for pin in pins_dir.iter_mut().chain(pins_step.iter_mut()) {
                pin.set_low();
            }

            for pin in pins_enable.iter_mut() {
                pin.set_high();
            }

            // set microstepping
            for motor_i in 0..4 {
                for value_i in 0..3 {
                    let pin = &mut pins_micstep[motor_i][value_i];
                    if pin_settings.micstep_vals[motor_i][value_i] {
                        pin.set_high();
                    } else {
                        pin.set_low();
                    }
                }
            }

            return Ok(Hardware {
                enabled: false,
                x: 0,
                y: 0,
                head_down: false,
                last_steps_timestamp: Map::from([
                    (Axis::X, Instant::now()),
                    (Axis::Y, Instant::now()),
                    (Axis::Head, Instant::now()),
                ]),
                #[cfg(feature = "raspi")]
                pins_dir,
                #[cfg(feature = "raspi")]
                pins_step,
                #[cfg(feature = "raspi")]
                pins_enable,
                #[cfg(feature = "raspi")]
                _pins_micstep: pins_micstep,
                pin_settings,
            });
        }
        #[cfg(not(feature = "raspi"))]
        {
            Ok(Hardware {
                enabled: false,
                x: 0,
                y: 0,
                head_down: false,
                last_steps_timestamp: Map::from([
                    (Axis::X, Instant::now()),
                    (Axis::Y, Instant::now()),
                    (Axis::Head, Instant::now()),
                ]),
                pin_settings,
            })
        }
    }

    pub fn get_pos(&self) -> V2 {
        V2::new(
            self.x as f32 * self.pin_settings.dist_per_step_axis_cm,
            self.y as f32 * self.pin_settings.dist_per_step_axis_cm,
        )
    }

    #[cfg(not(feature = "raspi"))]
    fn set_dir(&mut self, _axis: Axis, _forward: bool) {}

    #[cfg(feature = "raspi")]
    fn set_dir(&mut self, axis: Axis, forward: bool) {
        match axis {
            Axis::X => {
                if forward {
                    self.pins_dir[2].set_low();
                } else {
                    self.pins_dir[2].set_high();
                };
            }
            Axis::Y => {
                if forward {
                    self.pins_dir[0].set_low();
                    self.pins_dir[1].set_low();
                } else {
                    self.pins_dir[0].set_high();
                    self.pins_dir[1].set_high();
                };
            }
            Axis::Head => {
                if forward {
                    self.pins_dir[3].set_high();
                } else {
                    self.pins_dir[3].set_low();
                }
            }
        }
        sleep(Duration::new(0, PIN_DELAY_NANOS));
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

        #[cfg(feature = "raspi")]
        {
            match axis {
                Axis::X => {
                    self.pins_step[2].set_low();
                    sleep(Duration::new(0, PIN_DELAY_NANOS));
                    self.pins_step[2].set_high();
                }
                Axis::Y => {
                    self.pins_step[0].set_low();
                    self.pins_step[1].set_low();
                    sleep(Duration::new(0, PIN_DELAY_NANOS));
                    self.pins_step[0].set_high();
                    self.pins_step[1].set_high();
                }
                Axis::Head => {
                    self.pins_step[3].set_low();
                    sleep(Duration::new(0, PIN_DELAY_NANOS));
                    self.pins_step[3].set_high();
                }
            }
        }

        self.last_steps_timestamp.insert(axis, Instant::now());
    }

    fn move_steps(
        &mut self,
        movement: &V2i,
        speed_handler: &SpeedDelayHandler,
        speed_fraction_start: f32,
        speed_fraction_end: f32,
    ) {
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
            let moved_fraction = (stepped_x as f32 + stepped_y as f32) / total_steps as f32;
            let speed_fraction =
                speed_fraction_start + (speed_fraction_end - speed_fraction_start) * moved_fraction;

            if Line::new(V2::new(0.0, 0.0), movement_abs.abs().to_float())
                .point_relation(&V2::new(stepped_x as f32, stepped_y as f32))
                == PointLineRelation::Left
            {
                stepped_x += 1;
                self.x += directions_signs.x;
                self.step(Axis::X, speed_handler, speed_fraction);
            } else {
                stepped_y += 1;
                self.y += directions_signs.y;
                self.step(Axis::Y, speed_handler, speed_fraction);
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
            ((pos.point - self.get_pos()) / self.pin_settings.dist_per_step_axis_cm).round_to_int();
        if delta.is_zero() {
            return;
        }
        self.move_steps(&delta, speed_handler, speed_fraction_start, pos.speed);
    }

    // TODO: avoid mistakes with changing pen pressures
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

        self.set_dir(Axis::Head, down);
        let head_travel_cm = self.pin_settings.head_travel_to_touch_cm
            + self.pin_settings.extra_head_travel_for_pressure_cm * head_pressure;
        let head_travel_steps = self.pin_settings.steps_for_cm_head(head_travel_cm);

        for i in 0..head_travel_steps {
            let fraction = i as f32 / head_travel_steps as f32;
            let current_head_travel_cm = head_travel_cm * fraction;
            let speed_fraction_acc: f32 = current_head_travel_cm / accelleration_dist;
            let speed_fraction_decc: f32 =
                (head_travel_cm - current_head_travel_cm) / accelleration_dist;
            let speed_fraction = speed_fraction_acc.min(speed_fraction_decc).clamp(0.0, 1.0);

            self.step(Axis::Head, &speed_handler, speed_fraction);
        }

        self.head_down = down;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        #[cfg(feature = "raspi")]
        for pin in self.pins_enable.iter_mut() {
            if enabled {
                pin.set_low();
            } else {
                pin.set_high();
            }
        }
        self.enabled = enabled;
    }
}

impl Drop for Hardware {
    fn drop(&mut self) {
        self.set_enabled(false);
    }
}
