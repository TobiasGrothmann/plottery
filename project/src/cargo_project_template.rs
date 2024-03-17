use anyhow::Result;
use path_absolutize::Absolutize;
use rust_embed::RustEmbed;
use std::path::{Path, PathBuf};
use std::{fs::File, io::Write};
use vfs::{EmbeddedFS, FileSystem};

#[derive(RustEmbed, Debug)]
#[folder = "cargo_project_template/"]
struct CargoProjectTemplate;

fn get_fs() -> EmbeddedFS<CargoProjectTemplate> {
    EmbeddedFS::<CargoProjectTemplate>::new()
}

#[derive(Debug, Clone)]
pub enum LibSource {
    PlotteryHome,
    Path { path: PathBuf },
    Cargo,
}

struct StringReplacement {
    from: String,
    to: String,
}

pub fn generate_cargo_project_to_disk(
    out_dir: PathBuf,
    project_name: &str,
    lib_source: LibSource,
) -> Result<()> {
    let fs = get_fs();
    std::fs::create_dir_all(&out_dir)?;

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

    let string_replacements = vec![
        StringReplacement {
            from: "{{plottery-lib-include}}".to_string(),
            to: lib_include,
        },
        StringReplacement {
            from: "{{project-name}}".to_string(),
            to: project_name.to_owned(),
        },
    ];

    write_dir_to_disk_recurse(&fs, "".to_string(), &out_dir, &string_replacements)?;
    Ok(())
}

fn write_dir_to_disk_recurse(
    fs: &EmbeddedFS<CargoProjectTemplate>,
    sub_dir: String,
    out_dir: &PathBuf,
    string_replacements: &Vec<StringReplacement>,
) -> Result<()> {
    for element in fs.read_dir(&sub_dir).unwrap() {
        let sub_element = format!("{}/{}", &sub_dir, &element);
        let out_element = out_dir.join(sub_element.strip_prefix('/').unwrap()); // needs to be relative or else join replaces the whole path with sub_element
        if is_file(fs, &sub_element) {
            write_file_to_disk(fs, &sub_element, &out_element, string_replacements)?;
        } else {
            std::fs::create_dir_all(out_element)?;
            write_dir_to_disk_recurse(fs, sub_element.clone(), out_dir, string_replacements)?;
        }
    }
    Ok(())
}

fn write_file_to_disk(
    fs: &EmbeddedFS<CargoProjectTemplate>,
    sub_element: &str,
    out_element: &PathBuf,
    string_replacements: &Vec<StringReplacement>,
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

    let mut out_file = File::create(out_element)?;
    out_file.write_all(&buf)?;

    Ok(())
}

fn is_file(fs: &EmbeddedFS<CargoProjectTemplate>, sub_dir: &str) -> bool {
    fs.open_file(sub_dir).is_ok()
}
