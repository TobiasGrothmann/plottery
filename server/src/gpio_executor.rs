use plottery_server_lib::{hardware::HardwareExecutor, pins::PinSettings, Axis};
use std::time::{Duration, Instant};

#[cfg(feature = "raspi")]
use std::thread::sleep;

#[cfg(feature = "raspi")]
use rppal::gpio::{Gpio, OutputPin};

#[cfg(feature = "raspi")]
const PIN_DELAY_NANOS: u32 = 200;

#[derive(Debug)]
pub struct GpioExecutor {
    last_steps_timestamp: [Instant; 3],

    #[cfg(feature = "raspi")]
    pins_dir: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    pins_step: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    pins_enable: Vec<OutputPin>,
    #[cfg(feature = "raspi")]
    _pins_micstep: Vec<Vec<OutputPin>>,
}

impl GpioExecutor {
    pub fn new(_pin_settings: PinSettings) -> anyhow::Result<Self> {
        #[cfg(feature = "raspi")]
        {
            let gpio = Gpio::new()?;

            let mut pins_dir = Vec::with_capacity(4);
            let mut pins_step = Vec::with_capacity(4);
            let mut pins_enable = Vec::with_capacity(4);
            let mut pins_micstep = Vec::with_capacity(4);
            for i in 0..4 {
                pins_dir.push(gpio.get(_pin_settings.dir_pins[i])?.into_output());
                pins_step.push(gpio.get(_pin_settings.step_pins[i])?.into_output());
                pins_enable.push(gpio.get(_pin_settings.enable_pins[i])?.into_output());

                let mut micstep = Vec::with_capacity(3);
                for j in 0..3 {
                    micstep.push(gpio.get(_pin_settings.micstep_pins[j][i])?.into_output());
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
                    if _pin_settings.micstep_vals[motor_i][value_i] {
                        pin.set_high();
                    } else {
                        pin.set_low();
                    }
                }
            }

            return Ok(Self {
                last_steps_timestamp: [Instant::now(), Instant::now(), Instant::now()],
                #[cfg(feature = "raspi")]
                pins_dir,
                #[cfg(feature = "raspi")]
                pins_step,
                #[cfg(feature = "raspi")]
                pins_enable,
                #[cfg(feature = "raspi")]
                _pins_micstep: pins_micstep,
            });
        }

        #[cfg(not(feature = "raspi"))]
        {
            Ok(Self {
                last_steps_timestamp: [Instant::now(), Instant::now(), Instant::now()],
            })
        }
    }
}

impl HardwareExecutor for GpioExecutor {
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

    fn step(&mut self, axis: Axis) {
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

        #[cfg(not(feature = "raspi"))]
        println!("Step {:?}", axis);

        self.last_steps_timestamp[axis_to_index(axis)] = Instant::now();
    }

    fn wait_axis(&mut self, axis: Axis, delay_nanos: u32) {
        let axis_i = axis_to_index(axis);
        let delay = Duration::new(0, delay_nanos);
        let delay_until = self.last_steps_timestamp[axis_i] + delay;
        while delay_until >= Instant::now() {
            // wait
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        #[cfg(feature = "raspi")]
        for pin in self.pins_enable.iter_mut() {
            if enabled {
                pin.set_low();
            } else {
                pin.set_high();
            }
        }

        #[cfg(not(feature = "raspi"))]
        let _ = enabled;
    }
}

fn axis_to_index(axis: Axis) -> usize {
    match axis {
        Axis::X => 0,
        Axis::Y => 1,
        Axis::Head => 2,
    }
}
