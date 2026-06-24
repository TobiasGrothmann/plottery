use plottery_lib::V2;
use plottery_server_lib::{
    accelleration::speed_delay_handler::SpeedDelayHandler, hardware::Hardware, midi::midi_to_freq,
    pins::HARDWARE_CONSTS, plot_execution, plot_setting::PlotSettings, server_state::ServerState,
    task::Task,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;

use crate::gpio_executor::GpioExecutor;

pub async fn start_server(
    mut receiver: mpsc::Receiver<Task>,
    server_state: Arc<Mutex<ServerState>>,
) -> anyhow::Result<()> {
    let executor = GpioExecutor::new(HARDWARE_CONSTS.pin_settings)?;
    let mut hardware = Hardware::new(HARDWARE_CONSTS.hardware_profile, executor);
    sync_server_state(&server_state, &hardware, false);

    task::spawn(async move {
        while let Some(task) = receiver.recv().await {
            println!("...received task");
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
                Task::Abort => {
                    todo!()
                }
                Task::SetHead(head_down) => {
                    let settings = PlotSettings::default();
                    let speed_range = if head_down {
                        settings.speed_head_down.clone()
                    } else {
                        settings.speed_head_up.clone()
                    };

                    hardware.set_enabled(true);
                    sync_server_state(&server_state, &hardware, false);
                    hardware.set_head(
                        head_down,
                        settings.head_travel_beyond_paper_cm,
                        speed_range.accelleration_distance,
                        SpeedDelayHandler::new_from_speed_range(
                            &speed_range,
                            HARDWARE_CONSTS.hardware_profile.dist_per_step_head_cm,
                        ),
                    );
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
        }
    });

    Ok(())
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
