use anyhow::{Ok, Result};
use std::path::{Path, PathBuf};

use cargo_generate::{generate, GenerateArgs, TemplatePath};

pub fn generate_cargo_project(path: PathBuf, name: String) -> Result<()> {
    let mut path_to_template = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    path_to_template.pop(); // workspace dir
    path_to_template.push("templates/cargo_project_template");
    assert!(path_to_template.exists());

    let wasm_pack_args = GenerateArgs {
        destination: Some(path),
        name: Some(name),
        vcs: None,
        lib: true,
        template_path: TemplatePath {
            path: Some(
                path_to_template
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            ),
            ..TemplatePath::default()
        },
        ..GenerateArgs::default()
    };

    generate(wasm_pack_args)?;

    Ok(())
}
