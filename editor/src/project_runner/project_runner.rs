use dioxus_std::utils::rw::UseRw;

use plottery_project::{read_layer_from_stdout, Project};
use tokio::{
    sync::mpsc::error::TryRecvError,
    time::{self, Duration},
};

use crate::router_components::editor::LayerChangeWrapper;

#[derive(Clone)]
pub struct ProjectRunner {
    project: Project,
    pub cancel_tx: Option<tokio::sync::mpsc::Sender<()>>,
    layer_rw_output: UseRw<LayerChangeWrapper>,
}

impl ProjectRunner {
    pub fn new(project: Project, layer_rw_output: UseRw<LayerChangeWrapper>) -> Self {
        log::info!("Creating new ProjectRunner");
        Self {
            project,
            cancel_tx: None,
            layer_rw_output,
        }
    }

    pub fn trigger_run_project(&mut self, release: bool) {
        self.cancel_tx.take(); // cancels the previous run if it exists

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);
        let project = self.project.clone();

        let layer_rw = self.layer_rw_output.clone();
        log::info!("Spawning new task to run project");
        tokio::spawn(async move {
            let millis_sleep = 10;

            log::info!("Building...");
            let build_process = project.build_async(release);
            let mut build_process = match build_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error compiling project: {}", e);
                    return;
                }
            };

            // build while waiting for cancel signal
            loop {
                match cancel_rx.try_recv() {
                    Ok(_) => {
                        log::info!("Unknown error, build cancelled");
                        return;
                    }
                    Err(e) => match e {
                        tokio::sync::mpsc::error::TryRecvError::Empty => {}
                        tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                            nix::sys::signal::kill(
                                nix::unistd::Pid::from_raw(build_process.id() as i32),
                                nix::sys::signal::SIGTERM,
                            )
                            .unwrap();
                            build_process.kill().unwrap();
                            log::info!("Build killed - {:?}", e);
                            return;
                        }
                    },
                }
                match build_process.try_wait() {
                    Ok(Some(_)) => break, // process has terminated
                    Ok(None) => {}        // process is still running
                    Err(_) => return,     // process has failed
                };
                time::sleep(Duration::from_millis(millis_sleep)).await;
            }

            // run while waiting for cancel signal
            log::info!("Running...");
            let run_process = project.run_async(release);
            let mut run_process = match run_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error running project: {}", e);
                    return;
                }
            };

            loop {
                match cancel_rx.try_recv() {
                    Ok(_) => {
                        log::info!("Unknown error, run cancelled");
                        return;
                    }
                    Err(e) => match e {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            nix::sys::signal::kill(
                                nix::unistd::Pid::from_raw(run_process.id() as i32),
                                nix::sys::signal::SIGTERM,
                            )
                            .unwrap();
                            run_process.kill().unwrap();
                            log::info!("Run killed");
                            return;
                        }
                    },
                }
                match run_process.try_wait() {
                    Ok(Some(_)) => break, // process has terminated
                    Ok(None) => {}        // process is still running
                    Err(_) => return,     // process has failed
                };
                time::sleep(Duration::from_millis(millis_sleep)).await;
            }

            // getting layer from stdout of project
            log::info!("Reading layer from stdout...");
            let layer = read_layer_from_stdout(&mut run_process);
            let layer = match layer {
                Ok(layer) => layer,
                Err(e) => {
                    log::error!("Error reading layer from project: {}", e);
                    return;
                }
            };

            // Publishing Layer
            log::info!("Outputting layer...");
            let change_counter = layer_rw.read().unwrap().change_counter;
            let write_success: Result<(), dioxus_std::utils::rw::UseRwError> =
                layer_rw.write(LayerChangeWrapper {
                    layer: Some(layer),
                    change_counter: change_counter + 1,
                });
            if write_success.is_err() {
                log::error!("Error using generated Layer.");
                return;
            }
        });
        tokio::spawn(async move {}); // TODO: why is this needed?
    }
}
