use anyhow::{Ok, Result};
use std::{
    path::PathBuf,
    process::{Child, Command, ExitStatus},
};

pub fn compile_cargo_project(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<ExitStatus> {
    let mut child_process = compile_cargo_project_async(project_dir, target_dir, release)?;
    let build_status = child_process.wait()?;
    Ok(build_status)
}

pub fn compile_cargo_project_async(
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
