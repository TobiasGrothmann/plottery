mod discovery;
mod discovery_test;
mod params;
mod stats;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use futures_lite::future::block_on;
use plottery_project::{
    export_plottery_home, project_params_list_wrapper::ProjectParamsListWrapper, LibSource,
    Project, RunEvent,
};
use std::{
    future::pending,
    path::{Path, PathBuf},
};

use discovery::find_project;

fn parse_lib_source(value: &str) -> LibSource {
    match value {
        "cratesio" => LibSource::CratesIO,
        "home" => LibSource::PlotteryHome,
        path => LibSource::Path {
            path: PathBuf::from(path),
        },
    }
}

#[derive(Debug, Clone, Args)]
struct ProjectArg {
    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Project folder or .plottery file [default: found from the current folder upwards]"
    )]
    project: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
struct ParamArgs {
    #[arg(
        long,
        help = "Start from the code's defaults instead of the editor's saved values"
    )]
    defaults: bool,
    #[arg(
        long = "set",
        value_name = "NAME=VALUE",
        help = "Override a param for this command only, repeatable. Syntax as printed by `plottery params`, e.g. size=12.5, glow.radius=2, points.0=1,2"
    )]
    set: Vec<String>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    #[command(about = "Create a new project")]
    New {
        #[arg(help = "Folder in which the project folder is created")]
        path: String,
        #[arg(help = "Name of the project")]
        name: String,
        #[arg(
            value_name = "cratesio|home|PATH",
            short = 'l',
            long = "lib",
            help = "Plottery library source [default: cratesio]",
            long_help = "Plottery library source\n    cratesio [default]    Use the published crate from crates.io\n    home                  Use local source from PLOTTERY_HOME env var\n    PATH                  Use local source from a custom path"
        )]
        lib: Option<String>,
    },
    #[command(about = "Build the project and show cargo output")]
    Build {
        #[command(flatten)]
        project: ProjectArg,
    },
    #[command(about = "Build and run like the editor's Run button, updating the editor files")]
    Run {
        #[command(flatten)]
        project: ProjectArg,
    },
    #[command(
        about = "Print the params with the editor's saved values, one `--set` target per line"
    )]
    Params {
        #[command(flatten)]
        project: ProjectArg,
        #[command(flatten)]
        params: ParamArgs,
    },
    #[command(
        about = "Render to png and/or svg files and print drawing stats",
        long_about = "Render to png and/or svg files and print drawing stats.\n\nUses the editor's saved param values unless --defaults or --set say otherwise. These never change the saved values, except with --update-editor, which saves exactly what was rendered."
    )]
    Render {
        #[arg(
            required = true,
            value_name = "OUT",
            help = "Output files, the format follows the extension (.png or .svg)"
        )]
        outputs: Vec<PathBuf>,
        #[command(flatten)]
        project: ProjectArg,
        #[command(flatten)]
        params: ParamArgs,
        #[arg(
            long,
            default_value_t = 1600,
            help = "Longest side of png files in pixels"
        )]
        size: u32,
        #[arg(
            long,
            help = "Also save these params, the layer, preview and thumbnail for the editor"
        )]
        update_editor: bool,
    },
    #[command(about = "Set PLOTTERY_HOME, the local plottery checkout used by `new --lib home`")]
    SetHome {
        #[arg(help = "Path to the plottery workspace directory")]
        path: String,
    },
    #[command(about = "Print the plottery version")]
    Version,
}

const AFTER_HELP: &str = "\
Typical loop inside a project folder:
  plottery build                                  compile, show warnings
  plottery params                                 the user's param values
  plottery render /tmp/a.png --set size=12        render with an override, print stats";

#[derive(Parser, Debug)]
#[command(
    name = "plottery",
    version,
    about = "CLI to create, build and render plottery projects",
    long_about = None,
    after_help = AFTER_HELP
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

const RELEASE: bool = true;

#[derive(Debug, Clone, Copy, PartialEq)]
enum OutputFormat {
    Png,
    Svg,
}

/// Format from the extension, and the folder must exist, so mistakes show before building.
fn check_output(path: &Path) -> Result<OutputFormat> {
    let extension = path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase());
    let format = match extension.as_deref() {
        Some("png") => OutputFormat::Png,
        Some("svg") => OutputFormat::Svg,
        _ => bail!(
            "Can't tell the format of '{}', use a .png or .svg file",
            path.display()
        ),
    };
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        if !parent.is_dir() {
            bail!("Folder '{}' does not exist", parent.display());
        }
    }
    Ok(format)
}

fn print_event(event: RunEvent) {
    match event {
        RunEvent::StageRunning(stage) => eprintln!("{}...", stage),
        RunEvent::Output(line) => eprintln!("{}", line),
        RunEvent::StageStarting(_) | RunEvent::ParamsMerged(_) => {}
    }
}

fn load_editor_params(project: &Project) -> Result<ProjectParamsListWrapper> {
    project.load_editor_params().with_context(|| {
        format!(
            "Failed to read saved params '{}'",
            project.get_editor_params_path().display()
        )
    })
}

/// Builds, then returns the params to use (saved or defaults, plus `--set`s) and the defaults.
fn build_and_resolve_params(
    project: &Project,
    args: &ParamArgs,
) -> Result<(ProjectParamsListWrapper, ProjectParamsListWrapper)> {
    let overrides = args
        .set
        .iter()
        .map(|set| {
            set.split_once('=')
                .with_context(|| format!("--set '{set}' needs the form NAME=VALUE"))
        })
        .collect::<Result<Vec<_>>>()?;

    let defaults = block_on(project.build_and_get_params_async(RELEASE, print_event, pending()))?;
    let mut params = if args.defaults {
        defaults.clone()
    } else {
        ProjectParamsListWrapper::new_combined(&load_editor_params(project)?.list, &defaults.list)
    };
    for (name, value) in overrides {
        params.set(name, value)?;
    }
    Ok((params, defaults))
}

pub fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::New { name, path, lib } => {
            let lib_source = lib.as_deref().map_or(LibSource::CratesIO, parse_lib_source);
            let project = Project::new(PathBuf::from(path), &name);
            if project.exists() {
                bail!("Project already exists at '{}'", project.dir.display());
            }
            project
                .generate_to_disk(lib_source, true)
                .with_context(|| format!("Failed to create project '{}'", project.dir.display()))?;
            println!("Created project at '{}'", project.dir.display());
        }
        Command::Build { project } => {
            let project = find_project(project.project)?;
            block_on(project.build_async_streaming(RELEASE, &print_event, &mut pending()))?;
            println!("Built '{}'", project.dir.display());
        }
        Command::Run { project } => {
            let project = find_project(project.project)?;
            let (params, _) = build_and_resolve_params(
                &project,
                &ParamArgs {
                    defaults: false,
                    set: Vec::new(),
                },
            )?;
            let layer =
                block_on(project.run_with_params_async(RELEASE, &params, print_event, pending()))?;
            project.save_editor_outputs(&params, &layer)?;
            println!(
                "Updated editor files in '{}'",
                project.get_resource_dir().display()
            );
        }
        Command::Params { project, params } => {
            let project = find_project(project.project)?;
            let (params, defaults) = build_and_resolve_params(&project, &params)?;
            println!("{}", params::format_params(&params.list, &defaults.list));
        }
        Command::Render {
            outputs,
            project,
            params,
            size,
            update_editor,
        } => {
            let formats = outputs
                .iter()
                .map(|path| check_output(path))
                .collect::<Result<Vec<_>>>()?;
            let project = find_project(project.project)?;
            let (params, _) = build_and_resolve_params(&project, &params)?;
            let layer =
                block_on(project.run_with_params_async(RELEASE, &params, print_event, pending()))?;
            for (path, format) in outputs.into_iter().zip(formats) {
                let written = match format {
                    OutputFormat::Png => project.write_png(path.clone(), &layer, size),
                    OutputFormat::Svg => project.write_svg(path.clone(), &layer),
                };
                written.with_context(|| format!("Failed to write '{}'", path.display()))?;
                println!("Wrote '{}'", path.display());
            }
            if update_editor {
                project.save_editor_outputs(&params, &layer)?;
                println!(
                    "Updated editor files in '{}'",
                    project.get_resource_dir().display()
                );
            }
            println!("{}", stats::format_stats(&layer, &params.list));
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
        Command::Version => println!("plottery {}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
