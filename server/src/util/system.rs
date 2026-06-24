#[cfg(feature = "raspi")]
const CPU_RT_PRIORITY: i32 = 70;

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

    let sched_param = libc::sched_param {
        sched_priority: CPU_RT_PRIORITY,
    };

    let result = unsafe {
        libc::sched_setscheduler(
            0, // current calling thread
            libc::SCHED_FIFO,
            &sched_param as *const libc::sched_param,
        )
    };

    if result != 0 {
        let os_err = std::io::Error::last_os_error();
        eprintln!(
            "Failed to set CPU realtime scheduler to SCHED_FIFO priority {}: {}",
            CPU_RT_PRIORITY, os_err
        );
        std::process::abort();
    }
}
