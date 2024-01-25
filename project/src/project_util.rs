use anyhow::{anyhow, Ok, Result}; // Import the `anyhow` crate
use plottery_lib::Layer;
use std::{
    io::Read,
    path::PathBuf,
    process::{Child, Command, ExitStatus, Stdio},
};

pub fn build_cargo_project(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<ExitStatus> {
    let mut child_process = build_cargo_project_async(project_dir, target_dir, release)?;
    let build_status = child_process.wait()?;
    Ok(build_status)
}

pub fn build_cargo_project_async(
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

pub fn run_executable(path: &PathBuf) -> Result<Layer> {
    let mut child_process = run_executable_async(path)?;
    read_layer_from_stdout(&mut child_process)
}

pub fn run_executable_async(path: &PathBuf) -> Result<Child> {
    let child_process = Command::new(path)
        .args(["std-out"])
        .stdout(Stdio::piped())
        .spawn()?;
    Ok(child_process)
}

pub fn read_layer_from_stdout(child_process: &mut Child) -> Result<Layer> {
    let build_status = child_process.wait()?;

    if build_status.success() {
        let mut buf = Vec::new();
        child_process
            .stdout
            .as_mut()
            .unwrap()
            .read_to_end(&mut buf)?;
        Ok(Layer::new_from_binary(&buf)?)
    } else {
        Err(anyhow!("Failed to run executable")) // Use the `anyhow!` macro
    }
}
