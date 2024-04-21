use crate::router_components::editor::{LayerChangeWrapper, RunningState};
use dioxus::signals::{Readable, SyncSignal, Writable};
use plottery_lib::Layer;
use plottery_project::{read_object_from_stdout, Project, ProjectParam};

#[derive(Clone)]
pub struct ProjectRunner {
    project: Project,
    cancel_tx: Option<tokio::sync::mpsc::Sender<()>>,
    layer: SyncSignal<LayerChangeWrapper>,
}

impl ProjectRunner {
    pub fn new(project: Project, layer: SyncSignal<LayerChangeWrapper>) -> Self {
        log::info!("Creating new ProjectRunner");
        Self {
            project,
            cancel_tx: None,
            layer,
        }
    }

    pub fn trigger_run_project(
        &mut self,
        release: bool,
        mut running_state: SyncSignal<RunningState>,
        params: Vec<ProjectParam>,
    ) {
        self.cancel_tx.take(); // cancels the previous run if it exists

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);
        let project = self.project.clone();

        let mut layer_copy = self.layer;

        log::info!("Spawning new task to run project");
        tokio::spawn(async move {
            log::info!("starting build...");
            running_state.set(RunningState::StartingBuild {
                msg: "starting build".to_string(),
            });

            // build while waiting for cancel signal
            let build_process = project.build_async(release).await;
            let mut run_process = match build_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error compiling project: {}", e);
                    running_state.set(RunningState::BuildFailed {
                        msg: "starting build failed".to_string(),
                    });
                    return;
                }
            };

            log::info!("Building...");
            running_state.set(RunningState::Building {
                msg: "building".to_string(),
            });

            tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(run_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .expect("Failed to kill build process");
                    run_process.kill().expect("Failed to kill build process");
                    log::info!("Build killed");
                    running_state.set(RunningState::BuildKilled {
                        msg: "build killed".to_string(),
                    });
                    return;
                }
                build_status = run_process.status() => {
                    match build_status {
                        Ok(status) => {
                            if !status.success() {
                                let msg = format!("build failed (exit code: {})", status.code().unwrap_or(-1));
                                log::info!("{}", msg);
                                running_state.set(RunningState::BuildFailed {
                                    msg,
                                });
                                return;
                            }
                        }
                        Err(e) => {
                            log::error!("Error getting build status: {}", e);
                            running_state.set(RunningState::BuildFailed {
                                msg: "build failed (no status)".to_string(),
                            });
                            return;
                        }
                    }
                }
            }

            // run while waiting for cancel signal
            log::info!("starting run...");
            running_state.set(RunningState::StartingRun {
                msg: "starting run".to_string(),
            });

            let run_process = project.run_async(release, params).await;
            let mut run_process = match run_process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error running project: {}", e);
                    running_state.set(RunningState::RunFailed {
                        msg: "run failed".to_string(),
                    });
                    return;
                }
            };

            log::info!("running...");
            running_state.set(RunningState::Running {
                msg: "running".to_string(),
            });

            tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(run_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .expect("Failed to kill run process");
                    run_process.kill().expect("Failed to kill run process");

                    log::info!("run killed");
                    running_state.set(RunningState::RunKilled {
                        msg: "run killed".to_string(),
                    });
                }
                new_layer = read_object_from_stdout::<Layer>(&mut run_process) => {
                    // getting layer from stdout of project executable
                    let new_layer = match new_layer {
                        Ok(layer) => layer,
                        Err(e) => {
                            running_state.set(RunningState::RunFailed {
                                msg: "run failed".to_string(),
                            });
                            log::error!("Error receiving layer from project: {}", e);
                            return;
                        }
                    };

                    // Publishing Layer
                    log::info!("updating editor...");
                    running_state.set(RunningState::Updating {
                        msg: "updating editor".to_string(),
                    });

                    let change_counter = layer_copy.read().change_counter;
                    layer_copy.set(
                        LayerChangeWrapper {
                            layer: Some(new_layer),
                            change_counter: change_counter + 1,
                        }
                    );
                }
            }

            running_state.set(RunningState::Idle);
        });
        tokio::spawn(async move {}); // this is somehow needed to get tokio to execute the above task
    }
}
