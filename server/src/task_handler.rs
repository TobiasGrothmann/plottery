use plottery_lib::V2;
use plottery_server_lib::{
    accelleration::speed_delay_handler::SpeedDelayHandler, hardware::Hardware, midi::midi_to_freq,
    pins::HARDWARE_CONSTS, plot_execution, plot_setting::PlotSettings, server_state::ServerState,
    task::Task,
};
use std::sync::{atomic::AtomicBool, Arc, Mutex};
use std::thread;
use tokio::sync::mpsc;

use crate::gpio_executor::GpioExecutor;

pub fn start_server(
    mut receiver: mpsc::Receiver<Task>,
    server_state: Arc<Mutex<ServerState>>,
    abort: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let executor = GpioExecutor::new(HARDWARE_CONSTS.pin_settings)?;
    let mut hardware =
        Hardware::new(HARDWARE_CONSTS.hardware_profile, executor).with_abort_flag(abort);
    sync_server_state(&server_state, &hardware, false);

    thread::spawn(move || {
        #[cfg(feature = "raspi")]
        crate::util::system::prepare_plot_thread();

        while let Some(task) = receiver.blocking_recv() {
            println!("...received task");
            // An abort sent while idle must not cancel this task.
            hardware.clear_abort();
            match task {
                Task::Plot {
                    layer,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, true);

                    plot_execution::plot_layer(
                        &mut hardware,
                        &layer,
                        sample_settings,
                        &plot_settings,
                    );
                    plot_execution::travel_to(&mut hardware, V2::zero(), &plot_settings);

                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::PlotShape {
                    shape,
                    sample_settings,
                    plot_settings,
                } => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, true);

                    plot_execution::plot_shape(
                        &mut hardware,
                        &shape,
                        sample_settings,
                        &plot_settings,
                    );
                    plot_execution::travel_to(&mut hardware, V2::zero(), &plot_settings);

                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::SetEnabled(enabled) => {
                    hardware.set_enabled(enabled);
                    sync_server_state(&server_state, &hardware, false);
                }
                // Handled by the `/task` route, never queued.
                Task::Abort => {}
                Task::SetHead(head_down) => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    set_head(&mut hardware, head_down);
                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::MoveTo(pos, plot_setting) => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    plot_execution::travel_to(&mut hardware, pos, &plot_setting);
                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::Move(delta, plot_settings) => {
                    let target_pos = hardware.get_pos() + delta;

                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    plot_execution::travel_to(&mut hardware, target_pos, &plot_settings);
                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::NoteFreq {
                    axis,
                    frequency,
                    duration_s,
                } => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    hardware.play_freq(&axis, frequency, duration_s);
                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::Note {
                    axis,
                    midi,
                    duration_s,
                } => {
                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    let freq = midi_to_freq(midi);
                    hardware.play_freq(&axis, freq, duration_s);
                    hardware.set_enabled(false);
                    sync_server_state(&server_state, &hardware, false);
                }
                Task::SetOrigin() => {
                    hardware.set_origin();
                    sync_server_state(&server_state, &hardware, false);
                }
            }

            if hardware.aborted() {
                while receiver.try_recv().is_ok() {}
                hardware.clear_abort();
                hardware.set_enabled(true);
                set_head(&mut hardware, false);
                hardware.set_enabled(false);
                sync_server_state(&server_state, &hardware, false);
                println!("aborted");
            }

            let stats = hardware.executor_mut().take_stats();
            if stats.steps > 0 {
                println!("{}", stats.summary());
            }
        }
    });

    Ok(())
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

fn sync_server_state(
    server_state: &Arc<Mutex<ServerState>>,
    hardware: &Hardware<GpioExecutor>,
    plotting: bool,
) {
    let mut state = server_state
        .lock()
        .expect("Failed to acquire server state lock");
    state.location = hardware.get_pos();
    state.head_down = hardware.is_head_down();
    state.motors_enabled = hardware.is_enabled();
    state.plotting = plotting;
}
