use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicU64, Ordering};

/// Live machine state, written by the plot thread and read by any other thread without a lock.
#[derive(Debug, Default)]
pub struct Telemetry {
    x: AtomicI32,
    y: AtomicI32,
    steps: AtomicU64,
    head_down: AtomicBool,
    enabled: AtomicBool,
    shape: AtomicU32,
}

/// One read of [`Telemetry`]; `x` and `y` are in steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TelemetryReading {
    pub x: i32,
    pub y: i32,
    /// All X, Y and head steps since the server started.
    pub steps: u64,
    pub head_down: bool,
    pub enabled: bool,
    /// Index of the shape being plotted; the shape count once all are done.
    pub shape: u32,
}

impl Telemetry {
    pub(crate) fn store(&self, x: i32, y: i32, steps: u64, head_down: bool, enabled: bool) {
        self.x.store(x, Ordering::Relaxed);
        self.y.store(y, Ordering::Relaxed);
        self.steps.store(steps, Ordering::Relaxed);
        self.head_down.store(head_down, Ordering::Relaxed);
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    pub fn set_shape(&self, index: u32) {
        self.shape.store(index, Ordering::Relaxed);
    }

    pub fn read(&self) -> TelemetryReading {
        TelemetryReading {
            x: self.x.load(Ordering::Relaxed),
            y: self.y.load(Ordering::Relaxed),
            steps: self.steps.load(Ordering::Relaxed),
            head_down: self.head_down.load(Ordering::Relaxed),
            enabled: self.enabled.load(Ordering::Relaxed),
            shape: self.shape.load(Ordering::Relaxed),
        }
    }
}
