#[cfg(feature = "raspi")]
const PLOT_CORE: usize = 2;

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
