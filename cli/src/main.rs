use clap::Subcommand;
use clap::{Parser, ValueEnum};
use plottery_project::{LibSource, Project};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum RenderType {
    Svg,
    Png,
}

fn parse_lib_source(value: &str) -> LibSource {
    match value {
        "cratesio" => LibSource::CratesIO,
        "home" => LibSource::PlotteryHome,
        path => LibSource::Path {
            path: PathBuf::from(path),
        },
    }
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    New {
        #[arg(help = "Directory where the project will be created")]
        path: String,
        #[arg(help = "Name of the project")]
        name: String,
        #[arg(
            value_name = "cratesio|home|PATH",
            short = 'l',
            help = "Plottery library source [default: cratesio]",
            long_help = "Plottery library source\n    cratesio [default]    Use the published crate from crates.io\n    home                  Use local source from PLOTTERY_HOME env var\n    PATH                  Use local source from a custom path"
        )]
        lib: Option<String>,
    },
    Render {
        #[arg(help = "Output format (svg or png)")]
        format: RenderType,
        #[arg(help = "Path to the .plottery project file")]
        project_path: String,
        #[arg(help = "Output file path")]
        out_path: String,
    },
    Build {
        #[arg(help = "Path to the .plottery project file")]
        project_path: String,
    },
}

#[derive(Parser, Debug)]
#[command(about="CLI helper to create and manager plottery projects", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

pub fn main() {
    let args = Args::parse();
    match args.command {
        Command::New { name, path, lib } => {
            let lib_source = match lib {
                Some(value) => parse_lib_source(&value),
                None => LibSource::CratesIO,
            };

            let project = Project::new(PathBuf::from(path), &name);

            if project.exists() {
                println!(
                    "Project already exists at '{}'",
                    project
                        .dir
                        .to_str()
                        .expect("Failed to convert project directory path to string")
                );
                return;
            }

            let result = project.generate_to_disk(lib_source, true);
            if result.is_err() {
                println!("Failed to create project: {}", result.unwrap_err());
                return;
            }

            println!(
                "Created project at '{}'",
                project
                    .dir
                    .to_str()
                    .expect("Failed to convert project directory path to string")
            );
        }
        Command::Build { project_path } => {
            let project_path_buf = PathBuf::from(project_path);
            if project_path_buf.is_dir() {
                println!(
                    "Path '{}' is a directory. Use the '.plottery' file.",
                    project_path_buf.to_str().unwrap()
                );
                return;
            }

            let project = Project::load_from_file(project_path_buf).unwrap();
            if !project.exists() {
                println!("No project found at '{}'", project.dir.to_str().unwrap());
                return;
            }

            project.build(true).unwrap();
        }
        Command::Render {
            format,
            project_path,
            out_path,
        } => {
            let project_path_buf = PathBuf::from(project_path);
            if project_path_buf.is_dir() {
                println!(
                    "Path '{}' is a directory. Use the '.plottery' file.",
                    project_path_buf.to_str().unwrap()
                );
                return;
            }

            let project = Project::load_from_file(project_path_buf).unwrap();
            if !project.exists() {
                println!("No project found at '{}'", project.dir.to_str().unwrap());
                return;
            }

            let release = true;
            project.build(release).unwrap();

            let params = project
                .run_get_params(release)
                .expect("Failed to get params");

            let out_path_buf = PathBuf::from(out_path);
            match format {
                RenderType::Svg => {
                    project.write_svg(out_path_buf, release, &params).unwrap();
                }
                RenderType::Png => {
                    project.write_png(out_path_buf, release, &params).unwrap();
                }
            }
        }
    }
}
