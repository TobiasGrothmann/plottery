use anyhow::Result;
use async_process::{Child, Command, Stdio};
use bincode::{deserialize_from, serialize};
use futures_lite::io::{AsyncRead, BufReader};
use futures_lite::prelude::*;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use serde::de::DeserializeOwned;
use std::{path::PathBuf, process::ExitStatus};

use crate::project_params_list_wrapper::ProjectParamsListWrapper;

fn find_cargo_executable() -> Result<PathBuf> {
    let output = std::process::Command::new("which").arg("cargo").output()?;
    if output.status.success() {
        let cargo_output = String::from_utf8_lossy(&output.stdout).to_string();
        let cargo_path = cargo_output
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .map(PathBuf::from);

        if let Some(cargo_path) = cargo_path {
            if cargo_path.exists() {
                return Ok(cargo_path);
            }
        }
    }

    let fallback = PathBuf::from(std::env::var("HOME")?)
        .join(".cargo")
        .join("bin")
        .join("cargo");
    if fallback.exists() {
        return Ok(fallback);
    }

    Err(anyhow::anyhow!(
        "Cargo executable not found. Tried `which cargo` and {}",
        fallback.to_string_lossy()
    ))
}

pub async fn build_cargo_project_async(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<Child> {
    let cargo_executable = find_cargo_executable()?;

    let mut args = vec!["build".to_string()];
    if release {
        args.push("--release".to_string());
    }
    args.push("--target-dir".to_string());
    args.push(target_dir.to_string_lossy().to_string());

    let child_process = Command::new(cargo_executable)
        .args(args)
        .current_dir(project_dir)
        .stderr(Stdio::piped())
        .spawn()?;
    anyhow::Ok(child_process)
}

pub async fn run_project_executable_async(
    path: &PathBuf,
    arguments: &[&str],
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
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(params) = params {
        if let Some(mut stdin) = child_process.stdin.take() {
            let binary = serialize(&params)?;
            stdin.write_all(&binary).await?;
        } else {
            return Err(anyhow::Error::msg("Failed to open stdin on child process"));
        }
    }

    anyhow::Ok(child_process)
}

pub async fn read_object_from_stdout<T>(child_process: &mut Child) -> Result<T>
where
    T: DeserializeOwned,
{
    let mut buf = Vec::new();
    if let Some(stdout) = &mut child_process.stdout {
        (*stdout).read_to_end(&mut buf).await?;
    }
    anyhow::Ok(deserialize_from(buf.as_slice())?)
}

pub async fn read_stdout_as_string_to_end(
    child_process: &mut Child,
) -> Result<(ExitStatus, String)> {
    let mut data = String::new();
    if let Some(stdout) = &mut child_process.stdout {
        (*stdout).read_to_string(&mut data).await?;
    }
    Ok((child_process.status().await?, data))
}

pub async fn process_lines_to_end<R, F>(stream: R, mut line_handler: F) -> Result<()>
where
    R: AsyncRead + Unpin,
    F: FnMut(String),
{
    let mut reader = BufReader::new(stream).lines();
    while let Some(line) = reader.next().await {
        line_handler(line?);
    }
    Ok(())
}

pub async fn process_stdout_lines<F>(
    child_process: &mut Child,
    mut line_handler: F,
) -> Result<ExitStatus>
where
    F: FnMut(String),
{
    let stdout = child_process
        .stdout
        .take()
        .ok_or_else(|| anyhow::Error::msg("Failed to get stdout handle"))?;

    process_lines_to_end(stdout, &mut line_handler).await?;
    Ok(child_process.status().await?)
}

pub async fn process_stderr_lines<F>(
    child_process: &mut Child,
    mut line_handler: F,
) -> Result<ExitStatus>
where
    F: FnMut(String),
{
    let stderr = child_process
        .stderr
        .take()
        .ok_or_else(|| anyhow::Error::msg("Failed to get stderr handle"))?;

    process_lines_to_end(stderr, &mut line_handler).await?;
    Ok(child_process.status().await?)
}
