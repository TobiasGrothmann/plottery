use anyhow::{Ok, Result};
use std::{
    path::PathBuf,
    process::{Command, ExitStatus},
};

pub fn compile_cargo_project(dir: PathBuf, release: bool) -> Result<ExitStatus> {
    let build_type = if release { "--release" } else { "--debug" };
    let build_status = Command::new("cargo")
        .args(["build", build_type])
        .current_dir(dir)
        .status()?;
    Ok(build_status)
}
