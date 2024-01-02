use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

pub mod project_definition;
pub use project_definition::Project;
pub use project_definition::ProjectConfig;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum RenderType {
    Svg,
    Png,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    New {
        path: String,
        name: String,
    },
    Render {
        format: RenderType,
        project_path: String,
        out_path: String,
    },
    Compile {
        project_path: String,
    },
}

#[derive(Parser, Debug)]
#[command(about="CLI helper to create and manager plottery projects", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

pub fn main() {
    let args = Args::parse();
    match args.command {
        Commands::New { name, path } => {
            let project = Project::new(PathBuf::from(path), name.clone());

            if project.exists() {
                println!(
                    "Project already exists at '{}'",
                    project.dir.to_str().unwrap()
                );
                return;
            }

            let result = project.generate_to_disk();
            if result.is_err() {
                println!("Failed to create project: {}", result.unwrap_err());
                return;
            }

            println!("Created project at '{}'", project.dir.to_str().unwrap());
        }
        Commands::Compile { project_path } => {
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

            project.compile(true).unwrap();
        }
        Commands::Render {
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

            project.compile(true).unwrap();

            let out_path_buf = PathBuf::from(out_path);
            match format {
                RenderType::Svg => {
                    project.write_svg(&out_path_buf, true).unwrap();
                }
                RenderType::Png => {
                    project.write_png(&out_path_buf, true).unwrap();
                }
            }
        }
    }
}
