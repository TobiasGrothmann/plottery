use anyhow::{Ok, Result};
use std::{
    path::PathBuf,
    process::{Command, ExitStatus},
};

pub fn compile_cargo_project(
    project_dir: PathBuf,
    target_dir: PathBuf,
    release: bool,
) -> Result<ExitStatus> {
    let build_type = if release { "--release" } else { "--debug" };
    let build_status = Command::new("cargo")
        .args([
            "build",
            build_type,
            "--target-dir",
            target_dir.to_string_lossy().as_ref(),
        ])
        .current_dir(project_dir)
        .status()?;
    Ok(build_status)
}
