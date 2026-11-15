use std::sync::{Mutex, MutexGuard, PoisonError};

use super::Layer;

struct Recorder {
    every: usize,
    /// `snapshot` calls so far.
    calls: usize,
    on_frame: Box<dyn FnMut(Layer) + Send>,
}

static RECORDER: Mutex<Option<Recorder>> = Mutex::new(None);

fn recorder() -> MutexGuard<'static, Option<Recorder>> {
    RECORDER.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Adds `layer()` as the next frame of the animation that shows how the drawing comes about.
/// A frame is the whole picture at that moment, as if the run stopped there.
///
/// Does nothing unless the run asked for frames, as `plottery animate` does, and `layer` is only
/// called for frames that are kept. `layer` must not use the random helpers, or the drawing
/// changes when frames are asked for.
///
/// ### Example
/// ```
/// # use plottery_lib::*;
/// let mut points = vec![V2::new(0.0, 0.0)];
/// for step in 0..100 {
///     points.push(V2::polar(Angle::from_degrees(step as f32 * 10.0), step as f32 * 0.1));
///     snapshot(|| Layer::new_from([Path::new_from(points.clone())]));
/// }
/// ```
pub fn snapshot(layer: impl FnOnce() -> Layer) {
    // the lock is held until the frame is handed over, so frames stay in the order of the calls
    let mut recorder = recorder();
    let Some(recorder) = recorder.as_mut() else {
        return;
    };
    let call = recorder.calls;
    recorder.calls += 1;
    if call.is_multiple_of(recorder.every) {
        (recorder.on_frame)(layer());
    }
}

/// Makes [`snapshot`] hand every `every`-th frame to `on_frame`, starting with the first. Used by
/// the project runner when a run asks for frames.
pub fn record_frames(every: usize, on_frame: impl FnMut(Layer) + Send + 'static) {
    *recorder() = Some(Recorder {
        every: every.max(1),
        calls: 0,
        on_frame: Box::new(on_frame),
    });
}
