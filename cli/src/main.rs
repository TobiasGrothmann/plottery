use clap::Subcommand;
use clap::{Parser, ValueEnum};
use plottery_project::{LibSource, Project};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum RenderType {
    Svg,
    Png,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    New {
        path: String,
        name: String,
    },
    Render {
        format: RenderType,
        project_path: String,
        out_path: String,
    },
    Build {
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
        Command::New { name, path } => {
            let project = Project::new(PathBuf::from(path), &name);

            if project.exists() {
                println!(
                    "Project already exists at '{}'",
                    project.dir.to_str().unwrap()
                );
                return;
            }

            let result = project.generate_to_disk(LibSource::Cargo, true);
            if result.is_err() {
                println!("Failed to create project: {}", result.unwrap_err());
                return;
            }

            println!("Created project at '{}'", project.dir.to_str().unwrap());
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
