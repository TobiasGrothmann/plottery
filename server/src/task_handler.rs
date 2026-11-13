use plottery_lib::{Layer, SampleSettings, V2};
use plottery_server_lib::{
    accelleration::speed_delay_handler::SpeedDelayHandler, hardware::Hardware, midi::midi_to_freq,
    pins::HARDWARE_CONSTS, plot_execution, plot_setting::PlotSettings, task::Task, PlotTimeline,
    Telemetry,
};
use std::sync::{atomic::AtomicBool, Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Instant;
use tokio::sync::mpsc;

use crate::{gpio_executor::GpioExecutor, job::JobState};

pub fn start_server(
    mut receiver: mpsc::Receiver<Task>,
    telemetry: Arc<Telemetry>,
    job: Arc<Mutex<JobState>>,
    abort: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let executor = GpioExecutor::new(HARDWARE_CONSTS.pin_settings)?;
    let mut hardware = Hardware::new(HARDWARE_CONSTS.hardware_profile, executor)
        .with_abort_flag(abort)
        .with_telemetry(telemetry.clone());

    thread::spawn(move || {
        #[cfg(feature = "raspi")]
        crate::util::system::prepare_plot_thread();

        while let Some(task) = receiver.blocking_recv() {
            println!("...received task");
            // An abort sent while idle must not cancel this task.
            hardware.clear_abort();
            match &task {
                Task::Plot { .. } | Task::PlotShape { .. } => lock(&job).begin_plot(Instant::now()),
                _ => lock(&job).begin_task(),
            }

            match task {
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => plot(
                    &mut hardware,
                    &telemetry,
                    &job,
                    &layer,
                    sample_settings,
                    &plot_settings,
                ),
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => plot(
                    &mut hardware,
                    &telemetry,
                    &job,
                    &Layer::new_from([shape]),
                    sample_settings,
                    &plot_settings,
                ),
                Task::SetEnabled(enabled) => hardware.set_enabled(enabled),
                // Handled by the `/task` route, never queued.
                Task::Abort => {}
                Task::SetHead(head_down) => {
                    hardware.set_enabled(true);
                    set_head(&mut hardware, head_down);
                    hardware.set_enabled(false);
                }
                Task::MoveTo(pos, plot_setting) => {
                    hardware.set_enabled(true);
                    plot_execution::travel_to(&mut hardware, pos, &plot_setting);
                    hardware.set_enabled(false);
                }
                Task::Move(delta, plot_settings) => {
                    let target_pos = hardware.get_pos() + delta;

                    hardware.set_enabled(true);
                    plot_execution::travel_to(&mut hardware, target_pos, &plot_settings);
                    hardware.set_enabled(false);
                }
                Task::NoteFreq {
                    axis,
                    frequency,
                    duration_s,
                } => {
                    hardware.set_enabled(true);
                    hardware.play_freq(&axis, frequency, duration_s);
                    hardware.set_enabled(false);
                }
                Task::Note {
                    axis,
                    midi,
                    duration_s,
                } => {
                    hardware.set_enabled(true);
                    let freq = midi_to_freq(midi);
                    hardware.play_freq(&axis, freq, duration_s);
                    hardware.set_enabled(false);
                }
                Task::SetOrigin() => hardware.set_origin(),
            }

            let aborted = hardware.aborted();
            if aborted {
                while receiver.try_recv().is_ok() {}
                hardware.set_enabled(true);
                set_head(&mut hardware, false);
                hardware.set_enabled(false);
                // Cleared last, so the state reads "aborting" until the head is up.
                hardware.clear_abort();
                println!("aborted");
            }
            lock(&job).finish_task(aborted, Instant::now());

            let stats = hardware.executor_mut().take_stats();
            if stats.steps > 0 {
                println!("{}", stats.summary());
            }
        }
    });

    Ok(())
}

fn lock(job: &Mutex<JobState>) -> MutexGuard<'_, JobState> {
    job.lock().expect("Failed to acquire job state lock")
}

/// Estimates the plot from where the machine is, plots `layer` and returns to the origin.
fn plot(
    hardware: &mut Hardware<GpioExecutor>,
    telemetry: &Telemetry,
    job: &Mutex<JobState>,
    layer: &Layer,
    sample_settings: SampleSettings,
    plot_settings: &PlotSettings,
) {
    let prepare_start = Instant::now();
    let timeline = PlotTimeline::build_from(hardware, layer, sample_settings, plot_settings);
    if hardware.aborted() {
        return;
    }
    let shapes = timeline.shapes();
    println!(
        "plot pre-pass: took_s={:.3} shapes={} estimated_s={:.1}",
        prepare_start.elapsed().as_secs_f64(),
        shapes,
        timeline.total().as_secs_f64(),
    );

    telemetry.set_shape(0);
    lock(job).start_plot(timeline, hardware.steps());

    hardware.set_enabled(true);
    plot_execution::plot_layer(
        hardware,
        layer,
        sample_settings,
        plot_settings,
        |index, _| telemetry.set_shape(index as u32),
    );
    if !hardware.aborted() {
        telemetry.set_shape(shapes);
    }
    plot_execution::travel_to(hardware, V2::zero(), plot_settings);
    hardware.set_enabled(false);
}

/// Moves the head with the default head speeds.
fn set_head(hardware: &mut Hardware<GpioExecutor>, head_down: bool) {
    let settings = PlotSettings::default();
    let speed_range = if head_down {
        settings.speed_head_down
    } else {
        settings.speed_head_up
    };
    hardware.set_head(
        head_down,
        settings.head_travel_beyond_paper_cm,
        speed_range.accelleration_distance,
        SpeedDelayHandler::new_from_speed_range(
            &speed_range,
            HARDWARE_CONSTS.hardware_profile.dist_per_step_head_cm,
        ),
    );
}
