use std::sync::Arc;
use tokio::task::JoinHandle;

use plottery_project::Project;

#[derive(Debug, Clone)]
pub struct ProjectRunner {
    project: Project,
    handle: Option<Arc<JoinHandle<Result<(), ()>>>>,
    cancel_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl ProjectRunner {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            handle: None,
            cancel_tx: None,
        }
    }

    pub fn cancel(&mut self) {
        if let Some(cancel_tx) = &self.cancel_tx {
            match cancel_tx.blocking_send(()) {
                Ok(_) => {
                    log::info!("Cancel signal sent");
                }
                Err(e) => {
                    log::error!("Error sending cancel signal {}", e);
                }
            }
        }
        self.handle = None;
        self.cancel_tx = None;
    }

    pub fn trigger_run_project(&mut self, release: bool) {
        self.cancel();

        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        let project = self.project.clone();

        let handle = tokio::spawn(async move {
            let process = project.compile_async(release);
            let mut process = match process {
                Ok(process) => process,
                Err(e) => {
                    log::error!("Error compiling project {}", e);
                    return Err(());
                }
            };

            // compile while waiting for cancel signal
            loop {
                match cancel_rx.try_recv() {
                    Ok(_) => {
                        // cancel has been sent
                        nix::sys::signal::kill(
                            nix::unistd::Pid::from_raw(process.id() as i32),
                            nix::sys::signal::SIGTERM,
                        )
                        .unwrap();
                        process.kill().unwrap();
                        log::info!("Compilation killed");
                        return Err(());
                    }
                    Err(_) => {}
                }
                match process.try_wait() {
                    Ok(Some(_)) => break,     // process has terminated
                    Ok(None) => {}            // process is still running
                    Err(_) => return Err(()), // process has failed
                };
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            log::info!("Compile finished");

            // run while waiting for cancel signal
            let handle = tokio::spawn(async move { project.run_code(true) });
            loop {
                match cancel_rx.try_recv() {
                    Ok(_) => {
                        handle.abort();
                        log::info!("Run killed");
                        return Err(());
                    }
                    Err(_) => {}
                }
                if handle.is_finished() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
            let layer = handle.await.unwrap().unwrap();
            log::info!("Run finished - layer len {}", layer.len());

            Ok(())
        });

        self.handle = Some(Arc::new(handle));
        self.cancel_tx = Some(cancel_tx);
    }
}
