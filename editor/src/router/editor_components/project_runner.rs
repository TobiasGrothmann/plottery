use crate::router::{editor::LayerChangeWrapper, editor_components::running_state::RunningState};
use dioxus::signals::{Readable, SyncSignal, Writable};
use plottery_lib::Layer;
use plottery_project::{read_object_from_stdout, Project, ProjectParamsListWrapper};

use super::editor_console::EditorConsole;

#[derive(Clone)]
pub struct ProjectRunner {
    project: Project,
    cancel_tx: Option<tokio::sync::mpsc::Sender<()>>,
    layer: SyncSignal<LayerChangeWrapper>,
    params: SyncSignal<ProjectParamsListWrapper>,
}

impl ProjectRunner {
    pub fn new(
        project: Project,
        layer: SyncSignal<LayerChangeWrapper>,
        params: SyncSignal<ProjectParamsListWrapper>,
    ) -> Self {
        Self {
            project,
            cancel_tx: None,
            layer,
            params,
        }
    }

    pub fn trigger_run_project(
        &mut self,
        release: bool,
        mut running_state: SyncSignal<RunningState>,
        console: SyncSignal<EditorConsole>,
    ) {
        console.read().clear();
        self.cancel_tx.take(); // cancels the previous run if it exists

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);
        let project = self.project.clone();

        let mut layer_copy = self.layer;
        let mut params_copy = self.params;

        console.read().info("spawning new task to run project");
        tokio::spawn(async move {
            console.read().info("...starting build");
            running_state.set(RunningState::StartingBuild {
                msg: "starting build".to_string(),
            });

            // build while waiting for cancel signal
            let build_process = project.build_async(release).await;
            let mut run_process = match build_process {
                Ok(process) => process,
                Err(e) => {
                    console
                        .read()
                        .error(format!("Error compiling project: {}", e).as_str());
                    running_state.set(RunningState::BuildFailed {
                        msg: "starting build failed".to_string(),
                    });
                    return;
                }
            };

            console.read().info("...building");
            running_state.set(RunningState::Building {
                msg: "building".to_string(),
            });

            let success: bool = tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(run_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .expect("Failed to kill build process");
                    run_process.kill().expect("Failed to kill build process");
                    console.read().error("build killed");
                    running_state.set(RunningState::BuildKilled {
                        msg: "build killed".to_string(),
                    });
                    false
                }
                build_status = run_process.status() => {
                    let success: bool = match build_status {
                        Ok(status) => {
                            if !status.success() {
                                let msg = format!("build failed ({})", status.code().unwrap_or(-1));
                                console.read().error(msg.as_str());
                                running_state.set(RunningState::BuildFailed {
                                    msg,
                                });
                                false
                            } else {
                                true
                            }
                        }
                        Err(e) => {
                            console.read().error(format!("error getting build status: {}", e).as_str());
                            running_state.set(RunningState::BuildFailed {
                                msg: "build failed (no status)".to_string(),
                            });
                            false
                        }
                    };
                    success
                }
            };
            if !success {
                return;
            }

            // run get params while waiting for cancel signal
            console.read().info("...starting getting params");
            running_state.set(RunningState::StartingGetParams {
                msg: "starting getting params".to_string(),
            });

            let get_params_process = project.run_get_params_async(release).await;
            let mut get_params_process = match get_params_process {
                Ok(process) => process,
                Err(e) => {
                    console
                        .read()
                        .error(format!("error getting params from project: {}", e).as_str());
                    running_state.set(RunningState::GetParamsFailed {
                        msg: "get params failed".to_string(),
                    });
                    return;
                }
            };

            console.read().info("...getting params");
            running_state.set(RunningState::GetParams {
                msg: "getting params".to_string(),
            });

            let read_params = tokio::select! {
                _ = cancel_rx.recv() => {
                    nix::sys::signal::kill(
                        nix::unistd::Pid::from_raw(get_params_process.id() as i32),
                        nix::sys::signal::SIGTERM,
                    )
                    .expect("Failed to kill get params process");
                    run_process.kill().expect("Failed to kill get params process");

                    console.read().info("get params killed");
                    running_state.set(RunningState::GetParamsKilled {
                        msg: "get params killed".to_string(),
                    });
                    None
                }
                params_read = read_object_from_stdout::<ProjectParamsListWrapper>(&mut get_params_process) => {
                    // getting layer from stdout of project executable
                    match params_read {
                        Ok(params_read) => Some(params_read),
                        Err(e) => {
                            running_state.set(RunningState::RunFailed {
                                msg: "get params failed".to_string(),
                            });
                            console.read().error(format!("error receiving params from project: {}", e).as_str());
                            None
                        }
                    }
                }
            };
            if read_params.is_none() {
                return;
            }
            let read_params = read_params.unwrap();

            let new_params =
                ProjectParamsListWrapper::new_combined(&params_copy.read().list, &read_params.list);
            params_copy.set(new_params.clone());

            // run while waiting for cancel signal
            console.read().info("...starting run");
            running_state.set(RunningState::StartingRun {
                msg: "starting run".to_string(),
            });

            let run_process = project.run_async(release, &new_params).await;
            let mut run_process = match run_process {
                Ok(process) => process,
                Err(e) => {
                    console
                        .read()
                        .error(format!("Error running project: {}", e).as_str());
                    running_state.set(RunningState::RunFailed {
                        msg: "run failed".to_string(),
                    });
                    return;
                }
            };

            console.read().info("...running");
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

                    console.read().info("run killed");
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
                            console.read().error(format!("Error receiving layer from project: {}", e).as_str());
                            return;
                        }
                    };

                    // Publishing Layer
                    console.read().info("...updating editor");
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

            console.read().info("done");
            running_state.set(RunningState::Idle);
        });
        tokio::spawn(async move {}); // this is somehow needed to get tokio to execute the above task
    }
}
