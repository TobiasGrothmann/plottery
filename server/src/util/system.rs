#[cfg(feature = "raspi")]
const PLOT_CORE: usize = 2;
#[cfg(feature = "raspi")]
const PLOT_THREAD_FIFO_PRIORITY: libc::c_int = 50;

/// Everything the plot thread does to itself before its first task. Each
/// step that fails is logged and the loop carries on: it works on a shared
/// core too, only with more stalls.
///
/// `SCHED_FIFO` is only set when the core is isolated (`isolcpus=2` on the
/// kernel command line) and the realtime budget is unlimited
/// (`kernel.sched_rt_runtime_us = -1`). The step loop spins, so a realtime
/// thread on a shared core starves everything else there, and with the
/// default budget the kernel throttles it for 50 ms every second, which
/// is a hiccup once per second.
#[cfg(feature = "raspi")]
pub fn prepare_plot_thread() {
    if let Err(err) = set_performance_governor() {
        eprintln!("Could not set the performance cpufreq governor: {err}");
    }
    if let Err(err) = pin_this_thread_to_plot_core() {
        eprintln!("Could not pin the plot thread to core {PLOT_CORE}: {err}");
        return;
    }
    if !plot_core_is_isolated() {
        eprintln!(
            "Core {PLOT_CORE} is not isolated (no isolcpus in cmdline.txt); \
             plot thread stays on the normal scheduler"
        );
        return;
    }
    if !rt_budget_is_unlimited() {
        eprintln!(
            "kernel.sched_rt_runtime_us is not -1; plot thread stays on the normal scheduler"
        );
        return;
    }
    match set_this_thread_fifo() {
        Ok(()) => println!(
            "Plot thread on isolated core {PLOT_CORE} with SCHED_FIFO {PLOT_THREAD_FIFO_PRIORITY}"
        ),
        Err(err) => eprintln!("Could not set SCHED_FIFO on the plot thread: {err}"),
    }
}

/// Pin the calling thread to the plot core. `sched_setaffinity(0, ...)`
/// acts on the calling thread only (threads it spawns later inherit the
/// mask), so this runs inside the plot thread, never in `main`: pinning
/// `main` would drag tokio and Rocket onto the plot core as well.
#[cfg(feature = "raspi")]
fn pin_this_thread_to_plot_core() -> std::io::Result<()> {
    let mut cpu_set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    unsafe { libc::CPU_SET(PLOT_CORE, &mut cpu_set) };

    let result = unsafe {
        libc::sched_setaffinity(
            0,
            std::mem::size_of::<libc::cpu_set_t>(),
            &cpu_set as *const libc::cpu_set_t,
        )
    };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(feature = "raspi")]
fn set_this_thread_fifo() -> std::io::Result<()> {
    let param = libc::sched_param {
        sched_priority: PLOT_THREAD_FIFO_PRIORITY,
    };
    let result = unsafe { libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) };
    if result != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// True if the kernel was booted with `isolcpus` covering the plot core.
#[cfg(feature = "raspi")]
fn plot_core_is_isolated() -> bool {
    std::fs::read_to_string("/sys/devices/system/cpu/isolated")
        .map(|list| cpu_list_contains(list.trim(), PLOT_CORE))
        .unwrap_or(false)
}

#[cfg(feature = "raspi")]
fn rt_budget_is_unlimited() -> bool {
    std::fs::read_to_string("/proc/sys/kernel/sched_rt_runtime_us")
        .map(|value| value.trim() == "-1")
        .unwrap_or(false)
}

/// Parses the kernel's cpu list format: `2`, `1-3`, `0,2-3`.
#[cfg(feature = "raspi")]
fn cpu_list_contains(list: &str, cpu: usize) -> bool {
    list.split(',')
        .filter(|range| !range.is_empty())
        .any(|range| match range.split_once('-') {
            Some((low, high)) => matches!(
                (low.parse::<usize>(), high.parse::<usize>()),
                (Ok(low), Ok(high)) if (low..=high).contains(&cpu)
            ),
            None => range.parse::<usize>().ok() == Some(cpu),
        })
}

/// Keep the clock at maximum so the busy-wait timing does not wander with
/// frequency scaling. The Pi 3 has a single policy for all four cores.
#[cfg(feature = "raspi")]
fn set_performance_governor() -> std::io::Result<()> {
    std::fs::write(
        "/sys/devices/system/cpu/cpufreq/policy0/scaling_governor",
        "performance",
    )
}
