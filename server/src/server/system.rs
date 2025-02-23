#[cfg(all(target_os = "linux", target_arch = "arm"))]
pub fn set_realtime_priority() {
    use nix::sched::{sched_setscheduler, SchedParam, Scheduler};
    use nix::unistd::Pid;

    let param = SchedParam { sched_priority: 99 };
    sched_setscheduler(Pid::from_raw(0), Scheduler::Fifo, &param)
        .expect("Failed to set realtime priority");
}

#[cfg(not(all(target_os = "linux", target_arch = "arm")))]
pub fn set_realtime_priority() {
    println!("Realtime priority setting is not supported on this platform.");
}
