use anyhow::{Ok, Result};
use std::{
    path::PathBuf,
    process::{Command, ExitStatus},
};

pub fn compile_cargo_project(dir: PathBuf) -> Result<ExitStatus> {
    let build_status = Command::new("cargo")
        .args(["build"])
        .current_dir(dir)
        .status()?;
    Ok(build_status)
}
