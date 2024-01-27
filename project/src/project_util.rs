use anyhow::{anyhow, Ok, Result}; // Import the `anyhow` crate
use async_process::{Child, Command, Stdio};
use futures_lite::AsyncReadExt;
use plottery_lib::Layer;
use std::path::PathBuf;

pub async fn build_cargo_project_async(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<Child> {
    let build_type = if release { "--release" } else { "--debug" };
    let child_process = Command::new("cargo")
        .args([
            "build",
            build_type,
            "--target-dir",
            target_dir.to_string_lossy().as_ref(),
        ])
        .current_dir(project_dir)
        .spawn()?;
    Ok(child_process)
}

pub async fn run_executable_async(path: &PathBuf) -> Result<Child> {
    let child_process = Command::new(path)
        .args(["std-out"])
        .stdout(Stdio::piped())
        .spawn()?;
    Ok(child_process)
}

pub async fn read_layer_from_stdout(child_process: &mut Child) -> Result<Layer> {
    let build_status = child_process.status().await?;

    if build_status.success() {
        let mut buf = Vec::new();
        child_process
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut buf)
            .await?;
        Ok(Layer::new_from_binary(&buf)?)
    } else {
        Err(anyhow!("Failed to run executable")) // Use the `anyhow!` macro
    }
}
