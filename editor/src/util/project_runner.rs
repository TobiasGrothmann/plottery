use crate::router_components::editor::LayerChangeWrapper;
use dioxus::signals::{Readable, SyncSignal, Writable};
use plottery_project::{read_layer_from_stdout, Project};

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

    pub fn trigger_run_project(&mut self, release: bool) {
        self.cancel_tx.take(); // cancels the previous run if it exists

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        self.cancel_tx = Some(cancel_tx);
        let project = self.project.clone();

        let mut layer_clone = self.layer.clone();

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
                }
                new_layer = read_layer_from_stdout(&mut run_process) => {
                    // getting layer from stdout of project executable
                    let new_layer = match new_layer {
                        Ok(layer) => layer,
                        Err(e) => {
                            log::error!("Error reading layer from project: {}", e);
                            return;
                        }
                    };

                    // Publishing Layer
                    log::info!("Outputting layer...");
                    let change_counter = layer_clone.read().change_counter;
                    layer_clone.set(
                        LayerChangeWrapper {
                            layer: Some(new_layer),
                            change_counter: change_counter + 1,
                        }
                    );
                }
            }
        });
        tokio::spawn(async move {}); // this is somehow needed to get tokio to execute the above task
    }
}
