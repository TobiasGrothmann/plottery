use anyhow::Result;
use path_absolutize::Absolutize;
use std::path::{Path, PathBuf};

use cargo_generate::{generate, GenerateArgs, TemplatePath};

#[derive(Debug, Clone)]
pub enum LibSource {
    PlotteryHome,
    Path { path: PathBuf },
    Cargo,
}

pub fn generate_cargo_project(path: PathBuf, name: String, lib_source: LibSource) -> Result<()> {
    let mut path_to_template = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    path_to_template.push("cargo_project_template");
    assert!(path_to_template.exists());

    let lib_include = match lib_source {
        LibSource::PlotteryHome => {
            let plottery_home = match std::env::var("PLOTTERY_HOME") {
                Ok(home) => Path::new(&home).to_path_buf(),
                Err(_) => {
                    return Err(anyhow::anyhow!(
                        "Environment variable PLOTTERY_HOME is not set"
                    ));
                }
            };
            let plottery_home_lib = plottery_home
                .join("lib")
                .absolutize()
                .unwrap()
                .to_path_buf();
            if !plottery_home_lib.exists() {
                return Err(anyhow::anyhow!(
                    "PLOTTERY_HOME/lib does not exist at: {}",
                    plottery_home_lib.to_string_lossy()
                ));
            }
            let path = plottery_home_lib
                .absolutize()
                .unwrap()
                .to_string_lossy()
                .to_string();
            format!("plottery_lib = {{ path = \"{}\" }}", path)
        }
        LibSource::Path { path } => {
            let path = path.to_string_lossy().to_string();
            format!("plottery_lib = {{ path = \"{}\" }}", path)
        }
        LibSource::Cargo => "plottery_lib = \"0.*\"".to_string(),
    };

    let gen_args = GenerateArgs {
        destination: Some(path),
        name: Some(name),
        vcs: None,
        lib: true,
        define: vec![format!("plottery-lib-include={}", lib_include)],
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

    generate(gen_args)?;

    Ok(())
}
