use dioxus_std::utils::rw::UseRw;

use crate::router_components::editor::LayerChangeWrapper;
use plottery_project::{read_layer_from_stdout, Project};

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
            log::info!("Building...");

            // build while waiting for cancel signal
            let build_process = project.build_async(release).await;
            let mut run_process = match build_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error compiling project: {}", e);
                    return;
                }
            };

            tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(run_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .unwrap();
                    run_process.kill().unwrap();
                    log::info!("Build killed");
                    return;
                }
                build_status = run_process.status() => {
                    match build_status {
                        Ok(status) => {
                            if status.success() {
                                log::info!("Build successful");
                            } else {
                                log::info!("Build failed");
                                return;
                            }
                        }
                        Err(e) => {
                            log::error!("Error getting build status: {}", e);
                            return;
                        }
                    }
                }
            }

            // run while waiting for cancel signal
            log::info!("Running...");
            let run_process = project.run_async(release).await;
            let mut run_process = match run_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error running project: {}", e);
                    return;
                }
            };

            tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(run_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .unwrap();
                    run_process.kill().unwrap();
                    log::info!("Run killed");
                    return;
                }
                run_status = run_process.status() => {
                    match run_status {
                        Ok(status) => {
                            if status.success() {
                                log::info!("Run successful");
                            } else {
                                log::info!("Run failed");
                                return;
                            }
                        }
                        Err(e) => {
                            log::error!("Error getting run status: {}", e);
                            return;
                        }
                    }
                }
            }

            // getting layer from stdout of project executable
            log::info!("Reading layer from stdout...");
            let layer = read_layer_from_stdout(&mut run_process).await;
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
            }
        });
        tokio::spawn(async move {}); // this is somehow needed to get tokio to execute the above task
    }
}
