use plottery_server_lib::{hardware::HardwareExecutor, pins::PinSettings, Axis};
use std::time::{Duration, Instant};

#[cfg(feature = "raspi")]
use std::thread::sleep;

#[cfg(feature = "raspi")]
use rppal::gpio::{Gpio, OutputPin};

#[cfg(feature = "raspi")]
const PIN_DELAY_NANOS: u32 = 200;

/// A step whose *previous* step was commanded with a delay shorter than
/// this counts as "at speed": a pause there stalls a running motor. Below
/// that speed (`v_min` segments, shape boundaries while the next shape is
/// planned) a pause is harmless, so it is left out of the stall counters.
/// It still shows up in `gap_max`.
const AT_SPEED_DELAY: Duration = Duration::from_micros(800);
const STALL: Duration = Duration::from_micros(500);
const LONG_STALL: Duration = Duration::from_millis(5);

/// A partial summary is printed at most this often, and only right before
/// a wait of at least `REPORT_MIN_DELAY` (a corner or a shape boundary),
/// so the print itself cannot stall a motor that is running fast. It keeps
/// evidence of a long task even if the run is aborted.
const REPORT_INTERVAL: Duration = Duration::from_secs(10);
const REPORT_MIN_DELAY: Duration = Duration::from_millis(1);

/// Timing statistics of the busy-wait step loop, collected per task.
///
/// * `pulse`: how long `step()` itself takes (the GPIO pulse including any
///   sleep overhead). Its mean is the hidden per-step overhead that the
///   speed settings currently absorb.
/// * `gap`: time from the end of the previous step (on any axis) to the
///   start of this one, minus the previous step's commanded delay. Close to
///   zero in a healthy loop; large when the thread was not running.
/// * `spin`: time between two consecutive iterations of the wait loop. The
///   loop does nothing but read the clock, so a large gap means the thread
///   was preempted.
#[derive(Debug, Default, Clone, Copy)]
pub struct StepStats {
    pub steps: u64,
    pub first_step: Option<Instant>,
    pub last_step: Option<Instant>,
    pub pulse_sum: Duration,
    pub pulse_max: Duration,
    pub gap_max: Duration,
    pub at_speed_steps: u64,
    pub at_speed_gap_sum: Duration,
    pub at_speed_gap_max: Duration,
    pub stalls: u64,
    pub long_stalls: u64,
    pub spin_gap_max: Duration,
    pub spin_preemptions: u64,
}

impl StepStats {
    fn record_pulse(&mut self, start: Instant, end: Instant) {
        self.steps += 1;
        self.first_step.get_or_insert(start);
        self.last_step = Some(end);
        let pulse = end.saturating_duration_since(start);
        self.pulse_sum += pulse;
        self.pulse_max = self.pulse_max.max(pulse);
    }

    /// Wall time from the first to the last step of the task.
    pub fn span(&self) -> Duration {
        match (self.first_step, self.last_step) {
            (Some(first), Some(last)) => last.saturating_duration_since(first),
            _ => Duration::ZERO,
        }
    }

    fn record_gap(&mut self, gap: Duration, previous_delay: Duration) {
        self.gap_max = self.gap_max.max(gap);
        if previous_delay >= AT_SPEED_DELAY {
            return;
        }
        self.at_speed_steps += 1;
        self.at_speed_gap_sum += gap;
        self.at_speed_gap_max = self.at_speed_gap_max.max(gap);
        if gap > STALL {
            self.stalls += 1;
        }
        if gap > LONG_STALL {
            self.long_stalls += 1;
        }
    }

    #[inline]
    fn record_spin_gap(&mut self, gap: Duration) {
        if gap > self.spin_gap_max {
            self.spin_gap_max = gap;
        }
        if gap > STALL {
            self.spin_preemptions += 1;
        }
    }

    pub fn summary(&self) -> String {
        let us = |d: Duration| d.as_secs_f64() * 1e6;
        let mean = |sum: Duration, n: u64| if n == 0 { 0.0 } else { us(sum) / n as f64 };
        format!(
            "step stats: steps={} span_s={:.1} pulse_mean_us={:.1} pulse_max_us={:.1} \
             gap_max_us={:.1} at_speed_steps={} at_speed_gap_mean_us={:.1} \
             at_speed_gap_max_us={:.1} stalls_over_500us={} stalls_over_5ms={} \
             spin_preempt_max_us={:.1} spin_preempts_over_500us={}",
            self.steps,
            self.span().as_secs_f64(),
            mean(self.pulse_sum, self.steps),
            us(self.pulse_max),
            us(self.gap_max),
            self.at_speed_steps,
            mean(self.at_speed_gap_sum, self.at_speed_steps),
            us(self.at_speed_gap_max),
            self.stalls,
            self.long_stalls,
            us(self.spin_gap_max),
            self.spin_preemptions,
        )
    }
}

#[derive(Debug)]
pub struct GpioExecutor {
    last_steps_timestamp: [Instant; 3],
    /// Delay passed to the last `wait_axis` call per axis, i.e. the
    /// commanded delay of the step that follows it.
    pending_delay: [Duration; 3],
    /// End time and commanded delay of the last step on any axis within the
    /// current task. `None` at task start so the first step does not
    /// register a spurious gap.
    last_step: Option<(Instant, Duration)>,
    stats: StepStats,
    last_report: Instant,

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
                pending_delay: [Duration::ZERO; 3],
                last_step: None,
                stats: StepStats::default(),
                last_report: Instant::now(),
                pins_dir,
                pins_step,
                pins_enable,
                _pins_micstep: pins_micstep,
            });
        }

        #[cfg(not(feature = "raspi"))]
        {
            Ok(Self {
                last_steps_timestamp: [Instant::now(), Instant::now(), Instant::now()],
                pending_delay: [Duration::ZERO; 3],
                last_step: None,
                stats: StepStats::default(),
                last_report: Instant::now(),
            })
        }
    }

    /// Returns the statistics collected since the last call and starts a
    /// fresh collection. Call once per task.
    pub fn take_stats(&mut self) -> StepStats {
        self.last_step = None;
        self.last_report = Instant::now();
        std::mem::take(&mut self.stats)
    }

    fn maybe_report(&mut self, upcoming_delay: Duration) {
        if upcoming_delay < REPORT_MIN_DELAY || self.last_report.elapsed() < REPORT_INTERVAL {
            return;
        }
        println!("{} (partial)", self.stats.summary());
        self.last_report = Instant::now();
    }

    fn record_step(&mut self, axis: Axis, start: Instant, end: Instant) {
        let axis_i = axis_to_index(axis);
        self.stats.record_pulse(start, end);
        if let Some((previous_end, previous_delay)) = self.last_step {
            let gap = start
                .saturating_duration_since(previous_end)
                .saturating_sub(previous_delay);
            self.stats.record_gap(gap, previous_delay);
        }
        self.last_step = Some((end, self.pending_delay[axis_i]));
        self.last_steps_timestamp[axis_i] = end;
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
        let start = Instant::now();

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

        let end = Instant::now();
        self.record_step(axis, start, end);
    }

    fn wait_axis(&mut self, axis: Axis, delay_nanos: u32) {
        let axis_i = axis_to_index(axis);
        let delay = Duration::new(0, delay_nanos);
        self.pending_delay[axis_i] = delay;
        self.maybe_report(delay);
        let delay_until = self.last_steps_timestamp[axis_i] + delay;

        let mut previous = Instant::now();
        while delay_until >= previous {
            let now = Instant::now();
            self.stats
                .record_spin_gap(now.saturating_duration_since(previous));
            previous = now;
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
