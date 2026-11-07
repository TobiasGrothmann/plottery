#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use plottery_server_lib::{hardware::HardwareExecutor, pins::HARDWARE_CONSTS, Axis};

    use crate::gpio_executor::GpioExecutor;

    /// 100 µs: well inside the "at speed" range.
    const FAST: u32 = 100_000;
    /// 2 ms: a v_min segment, excluded from the stall counters.
    const SLOW: u32 = 2_000_000;

    fn executor() -> GpioExecutor {
        GpioExecutor::new(HARDWARE_CONSTS.pin_settings).expect("executor")
    }

    fn steps(executor: &mut GpioExecutor, axis: Axis, delay_nanos: u32, count: usize) {
        for _ in 0..count {
            executor.wait_axis(axis, delay_nanos);
            executor.step(axis);
        }
    }

    #[test]
    fn counts_steps_and_take_resets() {
        let mut executor = executor();
        steps(&mut executor, Axis::X, FAST, 10);
        let stats = executor.take_stats();
        assert_eq!(stats.steps, 10);
        // The first step has no previous step to measure a gap against.
        assert_eq!(stats.at_speed_steps, 9);
        assert_eq!(executor.take_stats().steps, 0);
    }

    #[test]
    fn stall_at_speed_is_counted() {
        let mut executor = executor();
        steps(&mut executor, Axis::X, FAST, 5);
        sleep(Duration::from_millis(3));
        steps(&mut executor, Axis::X, FAST, 5);
        let stats = executor.take_stats();
        assert!(stats.stalls >= 1, "{}", stats.summary());
        assert!(
            stats.at_speed_gap_max >= Duration::from_micros(2500),
            "{}",
            stats.summary()
        );
    }

    #[test]
    fn pause_at_slow_speed_is_visible_but_not_a_stall() {
        let mut executor = executor();
        steps(&mut executor, Axis::X, SLOW, 3);
        sleep(Duration::from_millis(3));
        steps(&mut executor, Axis::X, SLOW, 3);
        let stats = executor.take_stats();
        assert_eq!(stats.stalls, 0, "{}", stats.summary());
        assert_eq!(stats.at_speed_steps, 0);
        // The gap is measured beyond the commanded 2 ms delay, so a 3 ms
        // pause shows up as roughly 1 ms.
        assert!(
            stats.gap_max >= Duration::from_micros(900),
            "{}",
            stats.summary()
        );
    }

    #[test]
    fn first_step_of_a_task_has_no_gap() {
        let mut executor = executor();
        steps(&mut executor, Axis::X, FAST, 3);
        executor.take_stats();
        sleep(Duration::from_millis(3));
        steps(&mut executor, Axis::X, FAST, 1);
        let stats = executor.take_stats();
        assert_eq!(stats.steps, 1);
        assert_eq!(stats.stalls, 0, "{}", stats.summary());
        assert_eq!(stats.gap_max, Duration::ZERO);
    }

    #[test]
    fn interleaved_axes_do_not_count_as_gaps() {
        let mut executor = executor();
        for _ in 0..50 {
            executor.wait_axis(Axis::X, FAST);
            executor.step(Axis::X);
            executor.wait_axis(Axis::Y, FAST);
            executor.step(Axis::Y);
        }
        let stats = executor.take_stats();
        assert_eq!(stats.steps, 100);
        // Each step follows the other axis's step within its own delay, so
        // the measured gap is negative and clamps to zero. A stall can only
        // appear if the scheduler preempts the spin loop (the executor
        // detects that itself), so every stall must be explained by one.
        assert!(
            stats.stalls <= stats.spin_preemptions,
            "{}",
            stats.summary()
        );
    }

    #[test]
    fn pulse_duration_is_measured() {
        let mut executor = executor();
        steps(&mut executor, Axis::Head, FAST, 5);
        let stats = executor.take_stats();
        assert!(stats.pulse_sum > Duration::ZERO);
        assert!(stats.pulse_max >= stats.pulse_sum / 5);
        // Four waits of 100 µs separate the five steps.
        assert!(
            stats.span() >= Duration::from_micros(400),
            "{}",
            stats.summary()
        );
    }
}
