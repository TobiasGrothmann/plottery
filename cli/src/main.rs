mod animate;
mod discovery;
mod discovery_test;
mod editor_app;
mod params;
mod stats;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use futures_lite::future::block_on;
use plottery_project::{
    editor_roots, scan_roots, set_plottery_home, validate_project_name, LibSource, Project,
    RollEvent, RunEvent, ScanOptions, PLOTTERY_HOME_ENV,
};
use plottery_runtime::project_params_list_wrapper::ProjectParamsListWrapper;
use std::{
    cell::RefCell,
    future::{pending, poll_fn, Future},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    task::{Poll, Waker},
};

use animate::{VideoFormat, VideoOptions, VideoWriter};
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
        value_name = "PROJECT",
        help = "Project folder or .plottery file [default: found from the current folder upwards]"
    )]
    path: Option<PathBuf>,
    #[arg(
        short,
        long,
        value_name = "PATH",
        conflicts_with = "path",
        help = "Same as PROJECT"
    )]
    project: Option<PathBuf>,
}

impl ProjectArg {
    fn find(self) -> Result<Project> {
        find_project(self.path.or(self.project))
    }
}

/// `-p` only, for commands whose outputs are positional.
#[derive(Debug, Clone, Args)]
struct ProjectFlag {
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
            long_help = "Plottery library source\n    cratesio [default]    Use the published crate from crates.io\n    home                  Use the local checkout set with `plottery set-home`\n    PATH                  Use local source from a custom path"
        )]
        lib: Option<String>,
        #[arg(
            long,
            help = "Don't create a git repository and .gitignore in the project"
        )]
        no_git: bool,
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
    #[command(about = "Open the project in the Plottery Editor")]
    Open {
        #[command(flatten)]
        project: ProjectArg,
        #[arg(long, help = "Show its variations instead of the drawing")]
        variations: bool,
    },
    #[command(
        about = "Roll params to random values and store a drawing per roll, the project's variations",
        long_about = "Roll params to random values and store a drawing per roll, the project's variations.\n\nEach roll gives every named param a uniform random value: params with a #[range] within it, seeds from 100 to 999, angles over the full turn, bools, enums and options among their choices. Roll 0 is the start params themselves. The results replace the last ones in the project's variations/ folder (not committed): NNN.plotp, NNN.plotl and NNN.png per roll. The start params are the editor's saved values unless --defaults or --set say otherwise; the saved values don't change. The first failed run ends the roll, finished rolls stay. `plottery pick N` makes roll N the saved values."
    )]
    Roll {
        #[arg(
            value_name = "PARAM",
            help = "Params to roll, as named by `plottery params`, e.g. seed size glow.radius. Without any, lists the rollable ones"
        )]
        names: Vec<String>,
        #[arg(short = 'n', long, default_value_t = 24, help = "Number of rolls")]
        count: usize,
        #[command(flatten)]
        project: ProjectFlag,
        #[command(flatten)]
        params: ParamArgs,
        #[arg(
            long,
            help = "Open the variations in the editor, where the rolls show up as they finish"
        )]
        open: bool,
    },
    #[command(
        about = "Make a roll of `plottery roll` the saved values and its drawing the editor's",
        long_about = "Make a roll of `plottery roll` the saved values and its drawing the editor's, like `plottery set` and `plottery run` without running again.\n\nRoll 0 goes back to the params the rolls started from."
    )]
    Pick {
        #[arg(value_name = "N", help = "Number of the roll")]
        roll: usize,
        #[command(flatten)]
        project: ProjectFlag,
        #[arg(long, help = "Open the project in the editor afterwards")]
        open: bool,
    },
    #[command(
        about = "Print the folders of the projects the editor shows, one per line",
        long_about = "Print the folders of the projects the editor shows, one per line.\n\nThese are the projects in the folders added to the editor's browser. --roots prints those folders instead."
    )]
    List {
        #[arg(
            long,
            help = "Print the folders the editor scans instead of the projects"
        )]
        roots: bool,
    },
    #[command(
        about = "Print the path of a png of the project's last run, the editor's thumbnail",
        long_about = "Print the path of a png of the project's last run, the editor's thumbnail.\n\nNothing is built or run: it shows what `plottery run` or the editor drew last."
    )]
    Preview {
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
        about = "Change the user's saved param values, like editing them in the editor",
        long_about = "Change the user's saved param values, like editing them in the editor.\n\nAn open editor with hot reload on picks them up and reruns. Otherwise `plottery run` updates the editor's drawing."
    )]
    Set {
        #[arg(
            required_unless_present = "reset",
            value_name = "NAME=VALUE",
            help = "Syntax as printed by `plottery params`, e.g. size=12.5 glow.radius=2 points.0=1,2"
        )]
        assignments: Vec<String>,
        #[command(flatten)]
        project: ProjectFlag,
        #[arg(long, help = "Reset all params to the code's defaults first")]
        reset: bool,
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
        project: ProjectFlag,
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
    #[command(
        about = "Make an mp4 or gif of how the drawing comes about, from the project's `snapshot` calls",
        long_about = "Make an mp4 or gif of how the drawing comes about, from the project's `snapshot` calls.\n\nRuns the project once and asks it for its frames: every `snapshot(|| layer)` call in `generate` is one frame, the whole picture at that moment. Frames are rendered while the project runs and the video ends on the finished drawing. All frames show the paper, like a camera fixed above it, so the project needs a paper param. Params work as in `render`. Needs ffmpeg."
    )]
    Animate {
        #[arg(
            value_name = "OUT",
            help = "Output file, the format follows the extension (.mp4 or .gif)"
        )]
        output: PathBuf,
        #[command(flatten)]
        project: ProjectFlag,
        #[command(flatten)]
        params: ParamArgs,
        #[arg(
            long,
            default_value_t = 1600,
            help = "Longest side of the video in pixels"
        )]
        size: u32,
        #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u32).range(1..), help = "Frames per second")]
        fps: u32,
        #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..), value_name = "N", help = "Use only every N-th `snapshot`")]
        every: u32,
        #[arg(
            long,
            default_value_t = 2.0,
            value_name = "SECONDS",
            help = "How long the last frame stays at the end"
        )]
        hold: f32,
    },
    #[command(
        about = "Set the local plottery checkout used by `new --lib home`",
        long_about = "Set the local plottery checkout used by `new --lib home`.\n\nIt is saved in Plottery's config file, which the editor reads too. The PLOTTERY_HOME environment variable overrides it when set."
    )]
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
  plottery render /tmp/a.png --set size=12        render with an override, print stats
  plottery animate /tmp/a.mp4 --every 5           video of how the drawing comes about (`snapshot`)
  plottery roll seed size -n 24 --open            24 random variations, shown in the editor
  plottery pick 7                                 make roll 7 the user's values
  plottery set size=12                            change the user's values
  plottery run ~/art/labyrinth                    run a project from anywhere
  plottery open                                   open the project in the editor";

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

/// The lowercase extension of an output file of `command`. The file must not be a project and
/// its folder must exist, so mistakes show before building.
fn output_extension(path: &Path, command: &str) -> Result<Option<String>> {
    if path.is_dir() || path.extension().is_some_and(|ext| ext == "plottery") {
        bail!(
            "'{0}' is a project. Pass it with -p: plottery {command} -p {0} <OUT>...",
            path.display()
        );
    }
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        if !parent.is_dir() {
            bail!("Folder '{}' does not exist", parent.display());
        }
    }
    Ok(path
        .extension()
        .map(|ext| ext.to_string_lossy().to_lowercase()))
}

fn check_output(path: &Path) -> Result<OutputFormat> {
    match output_extension(path, "render")?.as_deref() {
        Some("png") => Ok(OutputFormat::Png),
        Some("svg") => Ok(OutputFormat::Svg),
        _ => bail!(
            "Can't tell the format of '{}', use a .png or .svg file",
            path.display()
        ),
    }
}

fn check_video_output(path: &Path) -> Result<VideoFormat> {
    let extension = output_extension(path, "animate")?;
    VideoFormat::from_extension(extension.as_deref()).with_context(|| {
        format!(
            "Can't tell the format of '{}', use a .mp4 or .gif file",
            path.display()
        )
    })
}

fn print_event(event: RunEvent) {
    match event {
        RunEvent::StageRunning(stage) => eprintln!("{}...", stage),
        RunEvent::Output(line) => eprintln!("{}", line),
        RunEvent::StageStarting(_) | RunEvent::ParamsMerged(_) | RunEvent::Snapshot(_) => {}
    }
}

/// The saved params. Names those whose saved value could not be read, they get their default.
fn load_editor_params(project: &Project) -> Result<ProjectParamsListWrapper> {
    let (params, skipped) = project.load_editor_params()?;
    for name in skipped {
        eprintln!("The saved value of '{name}' could not be read, using its default");
    }
    Ok(params)
}

fn parse_assignments(assignments: &[String]) -> Result<Vec<(&str, &str)>> {
    assignments
        .iter()
        .map(|assignment| {
            assignment
                .split_once('=')
                .with_context(|| format!("'{assignment}' needs the form NAME=VALUE"))
        })
        .collect()
}

/// Builds, then returns the params to use (saved or defaults, plus `--set`s) and the defaults.
fn build_and_resolve_params(
    project: &Project,
    args: &ParamArgs,
) -> Result<(ProjectParamsListWrapper, ProjectParamsListWrapper)> {
    let overrides = parse_assignments(&args.set)?;

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

/// The editor app, or an error that says how to get it.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);
static INTERRUPT_WAKER: Mutex<Option<Waker>> = Mutex::new(None);

/// Catches Ctrl-C from now on: the first one resolves the future, a second one exits.
fn interrupted() -> Result<impl Future<Output = ()>> {
    ctrlc::set_handler(|| {
        if INTERRUPTED.swap(true, Ordering::SeqCst) {
            std::process::exit(130);
        }
        if let Some(waker) = INTERRUPT_WAKER.lock().unwrap().take() {
            waker.wake();
        }
    })
    .context("Could not listen for Ctrl-C")?;
    Ok(poll_fn(|cx| {
        *INTERRUPT_WAKER.lock().unwrap() = Some(cx.waker().clone());
        match INTERRUPTED.load(Ordering::SeqCst) {
            true => Poll::Ready(()),
            false => Poll::Pending,
        }
    }))
}

fn check_editor_available() -> Result<PathBuf> {
    if !cfg!(target_os = "macos") {
        bail!("Opening the editor needs the Plottery Editor, which runs on macOS only");
    }
    editor_app::find_editor_app().context("Plottery Editor not found in /Applications or ~/Applications. Install it with: curl -fsSL https://raw.githubusercontent.com/TobiasGrothmann/plottery/main/scripts/install_macos.sh | bash")
}

/// Opens a project's `.plottery` file, or its `variations.json` to show the variations, in the editor.
fn open_in_editor(file: &Path) -> Result<()> {
    let app = check_editor_available()?;
    let file = std::path::absolute(file)?;
    let status = std::process::Command::new("open")
        .arg("-a")
        .arg(&app)
        .arg(&file)
        .status()
        .context("Failed to run `open`")?;
    if !status.success() {
        bail!("`open -a {}` failed ({status})", app.display());
    }
    Ok(())
}

pub fn main() -> Result<()> {
    let args = Cli::parse();
    match args.command {
        Command::New {
            name,
            path,
            lib,
            no_git,
        } => {
            validate_project_name(&name)?;
            let lib_source = lib.as_deref().map_or(LibSource::CratesIO, parse_lib_source);
            let project = Project::new(PathBuf::from(path), &name);
            if project.exists() {
                bail!("Project already exists at '{}'", project.dir.display());
            }
            project
                .generate_to_disk(lib_source, !no_git)
                .with_context(|| format!("Failed to create project '{}'", project.dir.display()))?;
            println!("Created project at '{}'", project.dir.display());
        }
        Command::Build { project } => {
            let project = project.find()?;
            block_on(project.build_async_streaming(RELEASE, &print_event, &mut pending()))?;
            println!("Built '{}'", project.dir.display());
        }
        Command::Run { project } => {
            let project = project.find()?;
            let (params, _) = build_and_resolve_params(
                &project,
                &ParamArgs {
                    defaults: false,
                    set: Vec::new(),
                },
            )?;
            let layer = block_on(project.run_with_params_async(
                RELEASE,
                &params,
                None,
                print_event,
                pending(),
            ))?;
            project.save_editor_outputs(&params, &layer)?;
            println!(
                "Ran '{}', the editor shows the new result",
                project.config.name
            );
            println!("{}", stats::format_stats(&layer, &params.list));
        }
        Command::Open {
            project,
            variations,
        } => {
            let project = project.find()?;
            let file = match variations {
                true => {
                    let meta = project.get_variations_meta_path();
                    if !meta.exists() {
                        bail!(
                            "'{}' has no variations yet, roll some first: plottery roll -p {} <PARAM>...",
                            project.config.name,
                            project.dir.display()
                        );
                    }
                    meta
                }
                false => project.get_project_config_path(),
            };
            open_in_editor(&file)?;
            println!("Opened '{}' in the editor", project.config.name);
        }
        Command::Roll {
            names,
            count,
            project,
            params,
            open,
        } => {
            if open {
                check_editor_available()?;
            }
            let project = find_project(project.project)?;
            let (start, _) = build_and_resolve_params(&project, &params)?;
            let outcome = block_on(project.roll_async(
                RELEASE,
                &start,
                &names,
                count,
                |event| match event {
                    RollEvent::Prepared { total } => {
                        eprintln!("Rolling, {total} runs...");
                        if open {
                            if let Err(e) = open_in_editor(&project.get_variations_meta_path()) {
                                eprintln!("Could not open the editor: {e}");
                            }
                        }
                    }
                    RollEvent::Finished { index, done, total } => {
                        eprintln!("{done}/{total} done (roll {index})")
                    }
                },
                interrupted()?,
            ));
            let stopped = INTERRUPTED.load(Ordering::SeqCst);
            // Ctrl-C reaches the running rolls too, which can end them as failed
            let outcome = match outcome {
                Err(e) if !stopped => return Err(e),
                outcome => outcome.ok(),
            };
            let variations = project
                .load_variations()?
                .context("The variations are gone")?;
            for roll in variations.rolls {
                let values = roll
                    .values
                    .iter()
                    .map(|(path, value)| format!("{path}={value}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let png = project.get_roll_path(roll.index, "png");
                println!("{:>3}  {values}  {}", roll.index, png.display());
            }
            let Some(outcome) = outcome.filter(|_| !stopped) else {
                bail!(
                    "Stopped. The finished rolls are in '{}'.",
                    project.get_variations_dir().display()
                );
            };
            println!(
                "Rolled '{}' {} times, the results are in '{}'. `plottery pick N` makes roll N the saved values.",
                project.config.name,
                outcome.total - 1,
                project.get_variations_dir().display()
            );
        }
        Command::Pick {
            roll,
            project,
            open,
        } => {
            if open {
                check_editor_available()?;
            }
            let project = find_project(project.project)?;
            let picked = project.load_roll_params(roll)?;
            let (before, defaults) = build_and_resolve_params(
                &project,
                &ParamArgs {
                    defaults: false,
                    set: Vec::new(),
                },
            )?;
            let after = ProjectParamsListWrapper::new_combined(&picked.list, &defaults.list);
            // a finished roll already has its drawing
            let layer = match project.load_roll_layer(roll) {
                Ok(layer) => layer,
                Err(_) => block_on(project.run_with_params_async(
                    RELEASE,
                    &after,
                    None,
                    print_event,
                    pending(),
                ))?,
            };
            project.save_editor_outputs(&after, &layer)?;
            println!(
                "{}",
                params::format_changes(&before.list, &after.list, &defaults.list)
            );
            println!("Picked roll {roll}, the editor shows it");
            if open {
                open_in_editor(&project.get_project_config_path())?;
            }
        }
        Command::List { roots: only_roots } => {
            let roots = editor_roots()?;
            if roots.is_empty() {
                bail!("The editor has no project folders yet. Add one in the editor's browser.");
            }
            if only_roots {
                for root in roots {
                    println!("{}", root.display());
                }
            } else {
                let scan = scan_roots(&roots, &ScanOptions::default());
                for (path, error) in scan.errors {
                    eprintln!("Skipped '{}': {error}", path.display());
                }
                let mut dirs: Vec<PathBuf> = scan.projects.into_iter().map(|p| p.dir).collect();
                dirs.sort();
                for dir in dirs {
                    println!("{}", dir.display());
                }
            }
        }
        Command::Preview { project } => {
            let thumbnail = project.find()?.fresh_editor_thumbnail()?;
            println!("{}", thumbnail.display());
        }
        Command::Params { project, params } => {
            let project = project.find()?;
            let (params, defaults) = build_and_resolve_params(&project, &params)?;
            println!("{}", params::format_params(&params.list, &defaults.list));
        }
        Command::Set {
            assignments,
            project,
            reset,
        } => {
            let assignments = parse_assignments(&assignments)?;
            let project = find_project(project.project)?;
            let (before, defaults) = build_and_resolve_params(
                &project,
                &ParamArgs {
                    defaults: false,
                    set: Vec::new(),
                },
            )?;
            let mut after = if reset {
                defaults.clone()
            } else {
                before.clone()
            };
            for (name, value) in assignments {
                after.set(name, value)?;
            }
            project.save_editor_params(&after)?;
            println!(
                "{}",
                params::format_changes(&before.list, &after.list, &defaults.list)
            );
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
            let layer = block_on(project.run_with_params_async(
                RELEASE,
                &params,
                None,
                print_event,
                pending(),
            ))?;
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
        Command::Animate {
            output,
            project,
            params,
            size,
            fps,
            every,
            hold,
        } => {
            let options = VideoOptions {
                format: check_video_output(&output)?,
                output: output.clone(),
                fps,
                hold: hold.max(0.0),
                size,
            };
            animate::check_ffmpeg()?;
            let project = find_project(project.project)?;
            let (params, _) = build_and_resolve_params(&project, &params)?;
            let Some(paper) = stats::find_paper(&params.list) else {
                bail!("The project has no paper param. Every frame shows the paper, and its size has to be known before the first frame arrives. Add a `Paper` param to `Params`.");
            };

            // frames are rendered as they arrive, the project waits meanwhile
            let video = RefCell::new(VideoWriter::new(options, paper.rect()));
            let failure = RefCell::new(None);
            let on_event = |event| match event {
                RunEvent::Snapshot(frame) if failure.borrow().is_none() => {
                    if let Err(e) = video.borrow_mut().push(frame) {
                        *failure.borrow_mut() = Some(e);
                    }
                }
                event => print_event(event),
            };
            // stops the run after a failed frame; looked at whenever the run waits for the project
            let cancel = poll_fn(|_| match failure.borrow().is_some() {
                true => Poll::Ready(()),
                false => Poll::Pending,
            });
            let run = block_on(project.run_with_params_async(
                RELEASE,
                &params,
                Some(every as usize),
                on_event,
                cancel,
            ));
            if let Some(e) = failure.into_inner() {
                return Err(e);
            }
            let layer = run?;

            let mut video = video.into_inner();
            if video.frame_count() == 0 {
                bail!("No frames: the project never calls `snapshot`. Call `snapshot(|| layer)` in `generate` for every state the animation should show.");
            }
            let stats = stats::format_stats(&layer, &params.list);
            // the video ends on the finished drawing, whatever the stride
            video.push(layer)?;
            let (frame_count, duration) = (video.frame_count(), video.duration());
            video.finish()?;
            println!(
                "Wrote '{}': {frame_count} frames, {duration:.1} s at {fps} fps",
                output.display()
            );
            println!("{stats}");
        }
        Command::SetHome { path } => {
            let path_buf = PathBuf::from(&path);
            if !path_buf.exists() {
                bail!("Path does not exist: {}", path);
            }
            if !path_buf.is_dir() {
                bail!("Path is not a directory: {}", path);
            }

            let home = path_buf.canonicalize().unwrap_or(path_buf);
            let config_path = set_plottery_home(&home).context("Failed to save plottery home")?;
            println!(
                "Plottery home set to '{}' in '{}'",
                home.display(),
                config_path.display()
            );
            if let Some(env_home) = std::env::var_os(PLOTTERY_HOME_ENV).filter(|v| !v.is_empty()) {
                if Path::new(&env_home) != home {
                    eprintln!(
                        "{PLOTTERY_HOME_ENV} is set to '{}' and overrides it. Remove it from your shell profile.",
                        env_home.to_string_lossy()
                    );
                }
            }
        }
        Command::Version => println!("plottery {}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
