#[cfg(feature = "raspi")]
const PLOT_CORE: usize = 2;

#[cfg(feature = "raspi")]
pub fn set_realtime_priority() {
    use ioprio::{set_priority, Pid, Priority, RtPriorityLevel, Target};

    if let Err(err) = set_priority(
        Target::ProcessGroup(Pid::this()),
        Priority::new(ioprio::Class::Realtime(RtPriorityLevel::highest())),
    ) {
        eprintln!("Failed to set IO realtime priority: {err}");
        std::process::abort();
    }

    // NOTE: No SCHED_FIFO here. The plotting loop busy-waits, which would
    // burn the RT runtime budget (sched_rt_runtime_us=950000 per 1s period)
    // and get throttled for the remainder of each period — causing regular
    // motor hiccups and lost steps.
}

/// Pin the whole process (all current and future threads) to a dedicated
/// core. Equivalent to `taskset -c 2`. Prevents the tokio/Rocket runtime
/// from being scheduled on other cores where periodic scheduler activity
/// would preempt the busy-wait step loop.
///
/// Must be called before the tokio runtime spawns its worker threads.
#[cfg(feature = "raspi")]
pub fn pin_process_to_plot_core() {
    let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    unsafe { libc::CPU_SET(PLOT_CORE, &mut cpu_set) };

    let result = unsafe {
        libc::sched_setaffinity(
            0, // 0 = calling process
            std::mem::size_of::<libc::cpu_set_t>(),
            &cpu_set as *const libc::cpu_set_t,
        )
    };

    if result != 0 {
        let os_err = std::io::Error::last_os_error();
        eprintln!(
            "Failed to pin process to CPU core {}: {}",
            PLOT_CORE, os_err
        );
        std::process::abort();
    }
}
