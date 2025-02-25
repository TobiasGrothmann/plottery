#[cfg(feature = "raspi")]
pub fn set_realtime_priority() {
    use ioprio::{set_priority, Pid, Priority, RtPriorityLevel, Target};

    set_priority(
        Target::ProcessGroup(Pid::this()),
        Priority::new(ioprio::Class::Realtime(RtPriorityLevel::highest())),
    )
    .unwrap();
}
