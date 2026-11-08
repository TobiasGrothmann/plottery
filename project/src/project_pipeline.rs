use std::future::Future;

use async_process::{Child, ChildStderr};
use futures_lite::future;
use plottery_lib::Layer;

use crate::{
    process_lines_to_end, process_stderr_lines, process_stdout_lines,
    project_params_list_wrapper::ProjectParamsListWrapper, read_object_from_stdout, Project,
    ProjectParam,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStage {
    Build,
    GetParams,
    Run,
}

impl std::fmt::Display for RunStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RunStage::Build => "build",
            RunStage::GetParams => "get params",
            RunStage::Run => "run",
        })
    }
}

#[derive(Debug, Clone)]
pub enum RunEvent {
    /// The stage's process is being spawned.
    StageStarting(RunStage),
    /// The stage's process is running.
    StageRunning(RunStage),
    /// A line of cargo or project output.
    Output(String),
    /// Project params merged with the saved ones, sent before the run starts.
    ParamsMerged(ProjectParamsListWrapper),
}

#[derive(Debug, Clone)]
pub enum RunError {
    Failed { stage: RunStage, msg: String },
    Killed { stage: RunStage },
}

impl RunError {
    fn failed(stage: RunStage, msg: String) -> Self {
        RunError::Failed { stage, msg }
    }
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::Failed { msg, .. } => f.write_str(msg),
            RunError::Killed { stage } => write!(f, "{} killed", stage),
        }
    }
}

impl std::error::Error for RunError {}

/// Runs `fut` until it finishes or `cancel` resolves, `None` when cancelled.
async fn until_cancelled<T>(
    fut: impl Future<Output = T>,
    cancel: &mut (impl Future<Output = ()> + Unpin),
) -> Option<T> {
    future::or(async { Some(fut.await) }, async {
        cancel.await;
        None
    })
    .await
}

fn kill_child(child: &mut Child) {
    let _ = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(child.id() as i32),
        nix::sys::signal::SIGTERM,
    );
    let _ = child.kill();
}

async fn forward_stderr(stderr: Option<ChildStderr>, on_event: &impl Fn(RunEvent)) {
    let Some(stderr) = stderr else {
        return;
    };
    if let Err(e) = process_lines_to_end(stderr, |line| on_event(RunEvent::Output(line))).await {
        on_event(RunEvent::Output(format!(
            "error receiving project stderr: {}",
            e
        )));
    }
}

impl Project {
    /// Builds the project, streaming cargo output as [`RunEvent::Output`].
    pub async fn build_async_streaming(
        &self,
        release: bool,
        on_event: &impl Fn(RunEvent),
        cancel: &mut (impl Future<Output = ()> + Unpin),
    ) -> Result<(), RunError> {
        let stage = RunStage::Build;
        on_event(RunEvent::StageStarting(stage));
        let mut child = self
            .build_async(release)
            .await
            .map_err(|e| RunError::failed(stage, format!("Error compiling project: {}", e)))?;

        on_event(RunEvent::StageRunning(stage));
        let status = until_cancelled(
            process_stderr_lines(&mut child, |line| on_event(RunEvent::Output(line))),
            cancel,
        )
        .await;

        match status {
            None => {
                kill_child(&mut child);
                Err(RunError::Killed { stage })
            }
            Some(Err(e)) => Err(RunError::failed(
                stage,
                format!("error getting build status: {}", e),
            )),
            Some(Ok(status)) if !status.success() => Err(RunError::failed(
                stage,
                format!("build failed ({})", status.code().unwrap_or(-1)),
            )),
            Some(Ok(_)) => Ok(()),
        }
    }

    /// Builds, gets params, merges them with `saved_params` and runs the project.
    pub async fn build_and_run_async(
        &self,
        release: bool,
        saved_params: &[ProjectParam],
        on_event: impl Fn(RunEvent),
        cancel: impl Future<Output = ()>,
    ) -> Result<(ProjectParamsListWrapper, Layer), RunError> {
        let mut cancel = std::pin::pin!(cancel);
        self.build_async_streaming(release, &on_event, &mut cancel)
            .await?;
        let params = self
            .get_params_stage(release, saved_params, &on_event, &mut cancel)
            .await?;
        on_event(RunEvent::ParamsMerged(params.clone()));
        let layer = self
            .run_stage(release, &params, &on_event, &mut cancel)
            .await?;
        Ok((params, layer))
    }

    /// Builds and returns the project's default params.
    pub async fn build_and_get_params_async(
        &self,
        release: bool,
        on_event: impl Fn(RunEvent),
        cancel: impl Future<Output = ()>,
    ) -> Result<ProjectParamsListWrapper, RunError> {
        let mut cancel = std::pin::pin!(cancel);
        self.build_async_streaming(release, &on_event, &mut cancel)
            .await?;
        self.get_params_stage(release, &[], &on_event, &mut cancel)
            .await
    }

    /// Runs the already built project with `params`.
    pub async fn run_with_params_async(
        &self,
        release: bool,
        params: &ProjectParamsListWrapper,
        on_event: impl Fn(RunEvent),
        cancel: impl Future<Output = ()>,
    ) -> Result<Layer, RunError> {
        let mut cancel = std::pin::pin!(cancel);
        self.run_stage(release, params, &on_event, &mut cancel)
            .await
    }

    async fn get_params_stage(
        &self,
        release: bool,
        saved_params: &[ProjectParam],
        on_event: &impl Fn(RunEvent),
        cancel: &mut (impl Future<Output = ()> + Unpin),
    ) -> Result<ProjectParamsListWrapper, RunError> {
        let stage = RunStage::GetParams;
        on_event(RunEvent::StageStarting(stage));
        let mut child = self.run_get_params_async(release).await.map_err(|e| {
            RunError::failed(stage, format!("error getting params from project: {}", e))
        })?;

        on_event(RunEvent::StageRunning(stage));
        let stderr = child.stderr.take();
        let read = until_cancelled(
            future::zip(
                read_object_from_stdout::<ProjectParamsListWrapper>(&mut child),
                forward_stderr(stderr, on_event),
            ),
            cancel,
        )
        .await;

        match read {
            None => {
                kill_child(&mut child);
                Err(RunError::Killed { stage })
            }
            Some((Err(e), _)) => Err(RunError::failed(
                stage,
                format!("error receiving params from project: {}", e),
            )),
            Some((Ok(params), _)) => Ok(ProjectParamsListWrapper::new_combined(
                saved_params,
                &params.list,
            )),
        }
    }

    async fn run_stage(
        &self,
        release: bool,
        params: &ProjectParamsListWrapper,
        on_event: &impl Fn(RunEvent),
        cancel: &mut (impl Future<Output = ()> + Unpin),
    ) -> Result<Layer, RunError> {
        let stage = RunStage::Run;
        on_event(RunEvent::StageStarting(stage));
        let (mut child, named_pipe) = self
            .run_async(release, params)
            .await
            .map_err(|e| RunError::failed(stage, format!("Error running project: {}", e)))?;

        on_event(RunEvent::StageRunning(stage));
        let stderr = child.stderr.take();
        let status = until_cancelled(
            future::zip(
                process_stdout_lines(&mut child, |line| on_event(RunEvent::Output(line))),
                forward_stderr(stderr, on_event),
            ),
            cancel,
        )
        .await;

        match status {
            None => {
                kill_child(&mut child);
                return Err(RunError::Killed { stage });
            }
            Some((Err(e), _)) => {
                return Err(RunError::failed(
                    stage,
                    format!("error getting running status: {}", e),
                ))
            }
            Some((Ok(status), _)) if !status.success() => {
                return Err(RunError::failed(
                    stage,
                    format!("run failed ({})", status.code().unwrap_or(-1)),
                ))
            }
            Some((Ok(_), _)) => {}
        }

        Layer::new_from_file(&named_pipe.path().to_path_buf())
            .map_err(|e| RunError::failed(stage, format!("error reading layer from run: {}", e)))
    }
}
