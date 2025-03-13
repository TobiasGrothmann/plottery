use anyhow::Result;
use path_absolutize::Absolutize;
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};
use std::vec;
use std::{fs::File, io::Write};
use vfs::{EmbeddedFS, FileSystem};

#[derive(RustEmbed, Debug)]
#[folder = "cargo_project_template/"]
struct CargoProjectTemplate;

static PLOTTERY_PROJECT_GITIGNORE: &str = include_str!("../resources/gitignore");

fn get_fs() -> EmbeddedFS<CargoProjectTemplate> {
    EmbeddedFS::<CargoProjectTemplate>::new()
}

#[derive(Debug, Clone)]
pub enum LibSource {
    PlotteryHome,
    Path { path: PathBuf },
    Cargo,
}

struct Replacement {
    from: String,
    to: String,
}

fn get_plottery_subcrate(
    plottery_home_subdir: &str,
    lib_source: &LibSource,
    crate_name: &str,
) -> Result<String> {
    let lib_source_for_toml = match lib_source {
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
                .join(plottery_home_subdir)
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
            format!("{} = {{ path = \"{}\" }}", crate_name, path)
        }
        LibSource::Path {
            path: plottery_home_path,
        } => {
            let sub_crate_path = plottery_home_path
                .join(plottery_home_subdir)
                .to_string_lossy()
                .to_string();
            format!("{} = {{ path = \"{}\" }}", crate_name, sub_crate_path)
        }
        LibSource::Cargo => format!("{} = \"^{}\"", crate_name, env!("CARGO_PKG_VERSION")),
    };
    Ok(lib_source_for_toml)
}

pub fn generate_cargo_project_to_disk(
    out_dir: PathBuf,
    project_name: &str,
    lib_source: LibSource,
) -> Result<()> {
    let fs = get_fs();
    std::fs::create_dir_all(&out_dir)?;

    let plottery_lib_include = get_plottery_subcrate("lib", &lib_source, "plottery_lib")?;
    let plottery_project_include =
        get_plottery_subcrate("project", &lib_source, "plottery_project")?;

    let string_replacements = vec![
        Replacement {
            from: "{{plottery-lib-include}}".to_string(),
            to: plottery_lib_include,
        },
        Replacement {
            from: "{{plottery-project-include}}".to_string(),
            to: plottery_project_include,
        },
        Replacement {
            from: "{{project-name}}".to_string(),
            to: project_name.to_owned(),
        },
    ];
    let file_name_replacements = vec![Replacement {
        from: "Cargo_template.toml".to_string(),
        to: "Cargo.toml".to_string(),
    }];

    write_dir_to_disk_recurse(
        &fs,
        "".to_string(),
        &out_dir,
        &string_replacements,
        &file_name_replacements,
    )?;

    Ok(())
}

fn write_dir_to_disk_recurse(
    fs: &EmbeddedFS<CargoProjectTemplate>,
    sub_dir: String,
    out_dir: &PathBuf,
    string_replacements: &[Replacement],
    file_name_replacements: &[Replacement],
) -> Result<()> {
    for element in fs.read_dir(&sub_dir).unwrap() {
        let sub_element = format!("{}/{}", &sub_dir, &element);
        let out_element = out_dir.join(sub_element.strip_prefix('/').unwrap()); // needs to be relative or else join replaces the whole path with sub_element
        if is_file(fs, &sub_element) {
            write_file_to_disk(
                fs,
                &sub_element,
                &out_element,
                string_replacements,
                file_name_replacements,
            )?;
        } else {
            std::fs::create_dir_all(out_element)?;
            write_dir_to_disk_recurse(
                fs,
                sub_element.clone(),
                out_dir,
                string_replacements,
                file_name_replacements,
            )?;
        }
    }
    Ok(())
}

fn write_file_to_disk(
    fs: &EmbeddedFS<CargoProjectTemplate>,
    sub_element: &str,
    out_element: &Path,
    string_replacements: &[Replacement],
    file_name_replacements: &[Replacement],
) -> Result<()> {
    let mut file = fs.open_file(sub_element)?;
    let ext = match Path::new(&sub_element).extension() {
        Some(ext) => ext.to_str().unwrap_or(""),
        None => "",
    };

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    if ext == "rs" || ext == "toml" {
        let mut buf_str = String::from_utf8(buf)?;
        for replacement in string_replacements {
            buf_str = buf_str.replace(&replacement.from, &replacement.to);
        }
        buf = buf_str.into_bytes();
    }

    let mut file_name = out_element
        .file_name()
        .expect("Failed to get file name")
        .to_str()
        .unwrap();

    for replacement in file_name_replacements {
        if file_name == replacement.from {
            file_name = &replacement.to;
            break;
        }
    }

    let out_file_path = out_element.with_file_name(file_name);

    let mut out_file = File::create(out_file_path)?;
    out_file.write_all(&buf)?;

    Ok(())
}

fn is_file(fs: &EmbeddedFS<CargoProjectTemplate>, sub_dir: &str) -> bool {
    fs.open_file(sub_dir).is_ok()
}

pub fn init_git_repo(dir: &PathBuf) -> Result<()> {
    let git_dir = dir.join(".git");
    if git_dir.exists() {
        return Ok(());
    }

    let output = std::process::Command::new("git")
        .arg("init")
        .arg(dir)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to initialize git repository at: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let gitignore_path = dir.join(".gitignore");
    if !gitignore_path.exists() {
        let mut gitignore_file = File::create(gitignore_path)?;
        gitignore_file.write_all(PLOTTERY_PROJECT_GITIGNORE.as_bytes())?;
    }

    Ok(())
}
