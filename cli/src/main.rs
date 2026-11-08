use anyhow::{bail, Context, Result};
use clap::Subcommand;
use clap::{Parser, ValueEnum};
use futures_lite::future::block_on;
use plottery_lib::Layer;
use plottery_project::{
    export_plottery_home, project_params_list_wrapper::ProjectParamsListWrapper, LibSource,
    Project, RunEvent,
};
use std::{future::pending, path::PathBuf};

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
    #[command(about = "Render with the editor's saved params to an svg or png file")]
    Render {
        #[arg(help = "Output format (svg or png)")]
        format: RenderType,
        #[arg(help = "Path to the .plottery project file")]
        project_path: String,
        #[arg(help = "Output file path")]
        out_path: String,
        #[arg(long, help = "Also update the editor preview, layer and thumbnail")]
        update_editor: bool,
    },
    #[command(about = "Build the project and show cargo output")]
    Build {
        #[arg(help = "Path to the .plottery project file")]
        project_path: String,
    },
    #[command(about = "Build and run like the editor's Run button, updating the editor files")]
    Run {
        #[arg(help = "Path to the .plottery project file")]
        project_path: String,
    },
    SetHome {
        #[arg(help = "Path to the plottery workspace directory")]
        path: String,
    },
}

#[derive(Parser, Debug)]
#[command(about="CLI helper to create and manager plottery projects", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

const RELEASE: bool = true;

fn load_project(project_path: &str) -> Result<Project> {
    let path = PathBuf::from(project_path);
    if path.is_dir() {
        bail!(
            "Path '{}' is a directory. Use the '.plottery' file.",
            path.display()
        );
    }
    Project::load_from_file(path)
}

fn print_event(event: RunEvent) {
    match event {
        RunEvent::StageRunning(stage) => eprintln!("{}...", stage),
        RunEvent::Output(line) => eprintln!("{}", line),
        RunEvent::StageStarting(_) | RunEvent::ParamsMerged(_) => {}
    }
}

/// Builds and runs with the editor's saved params.
fn build_and_run(project: &Project) -> Result<(ProjectParamsListWrapper, Layer)> {
    let saved_params = project.load_editor_params()?;
    Ok(block_on(project.build_and_run_async(
        RELEASE,
        &saved_params.list,
        print_event,
        pending(),
    ))?)
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::New { name, path, lib } => {
            let lib_source = lib.as_deref().map_or(LibSource::CratesIO, parse_lib_source);
            let project = Project::new(PathBuf::from(path), &name);
            if project.exists() {
                bail!("Project already exists at '{}'", project.dir.display());
            }
            project
                .generate_to_disk(lib_source, true)
                .context("Failed to create project")?;
            println!("Created project at '{}'", project.dir.display());
        }
        Command::Build { project_path } => {
            let project = load_project(&project_path)?;
            block_on(project.build_async_streaming(RELEASE, &print_event, &mut pending()))?;
        }
        Command::Run { project_path } => {
            let project = load_project(&project_path)?;
            let (params, layer) = build_and_run(&project)?;
            project.save_editor_outputs(&params, &layer)?;
            println!(
                "Updated editor files in '{}'",
                project.get_resource_dir().display()
            );
        }
        Command::Render {
            format,
            project_path,
            out_path,
            update_editor,
        } => {
            let project = load_project(&project_path)?;
            let (params, layer) = build_and_run(&project)?;
            let out_path = PathBuf::from(out_path);
            match format {
                RenderType::Svg => project.write_svg(out_path, &layer)?,
                RenderType::Png => project.write_png(out_path, &layer)?,
            }
            if update_editor {
                project.save_editor_outputs(&params, &layer)?;
            }
        }
        Command::SetHome { path } => {
            let path_buf = PathBuf::from(&path);
            if !path_buf.exists() {
                bail!("Path does not exist: {}", path);
            }
            if !path_buf.is_dir() {
                bail!("Path is not a directory: {}", path);
            }

            let absolute_path = path_buf.canonicalize().unwrap_or(path_buf);
            let absolute_path_str = absolute_path.to_string_lossy();

            export_plottery_home(&absolute_path_str).context("Failed to set PLOTTERY_HOME")?;
            println!("PLOTTERY_HOME has been set to: {}", absolute_path_str);
            println!("This setting will persist across terminal sessions.");
        }
    }
    Ok(())
}
