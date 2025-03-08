use anyhow::{Ok, Result};
use async_process::{Child, Command, Stdio};
use bincode::{deserialize_from, serialize};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

use crate::project_params_list_wrapper::ProjectParamsListWrapper;

pub async fn build_cargo_project_async(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<Child> {
    let mut args = vec!["build".to_string()];
    if release {
        args.push("--release".to_string());
    }
    args.push("--target-dir".to_string());
    args.push(target_dir.to_string_lossy().to_string());

    let child_process = Command::new("cargo")
        .args(args)
        .current_dir(project_dir)
        .spawn()?;
    Ok(child_process)
}

pub async fn run_project_executable_async(
    path: &PathBuf,                            // path to the executable
    arguments: &[&str],                        // arguments for the executable invocation
    params: Option<&ProjectParamsListWrapper>, // will be piped into stdin of the child process
) -> Result<Child> {
    let exec_stdin = if params.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };

    let mut child_process = Command::new(path)
        .args(arguments)
        .stdin(exec_stdin)
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(params) = params {
        if let Some(mut stdin) = child_process.stdin.take() {
            let binary = serialize(&params)?;
            stdin.write_all(&binary).await?;
        } else {
            return Err(anyhow::Error::msg("Failed to open stdin on child process"));
        }
    }

    Ok(child_process)
}

pub async fn read_object_from_stdout<T>(child_process: &mut Child) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut buf = Vec::new();
    if let Some(stdout) = &mut child_process.stdout {
        (*stdout).read_to_end(&mut buf).await?;
    }
    Ok(deserialize_from(buf.as_slice())?)
}
