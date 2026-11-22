use clap::{Parser, Subcommand};
use plottery_lib::{record_frames, Layer};
use std::error::Error;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::plottery_params::PlotteryParams;
use crate::project_params_list_wrapper::ProjectParamsListWrapper;
use crate::protocol::PROTOCOL_VERSION;
use crate::run_stream::{write_message, MessageKind};

#[derive(Debug, Clone, Subcommand)]
enum RunCommand {
    Svg {
        path: Option<String>,
    },
    StdOut {
        #[arg(long, short)]
        piped_params: Option<bool>,
    },
    /// Used by the editor and the CLI: sends the result through a Unix domain socket.
    Socket {
        /// Socket of the caller, the final layer and the frames are sent through it.
        path: PathBuf,
        /// Read the params from stdin instead of using the defaults.
        #[arg(long)]
        piped_params: bool,
        /// Also send every n-th frame the project passes to `snapshot`.
        #[arg(long)]
        frame_every: Option<usize>,
    },
    Dry {},
    Params {},
    /// Prints [`PROTOCOL_VERSION`], asked by the caller before anything else.
    ProtocolVersion {},
}

#[derive(Parser, Debug)]
#[command(
    about = "Plotter-project executable.",
    long_about = "Can be used to run the project and e.g. directly open as .svg."
)]
struct Args {
    #[command(subcommand)]
    command: RunCommand,
}

/// The params piped to stdin, or the defaults.
fn load_params<P: PlotteryParams>(piped: bool) -> Result<P, Box<dyn Error>> {
    if !piped {
        return Ok(P::try_new_from_list(P::param_defaults_list())?);
    }
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    let params: ProjectParamsListWrapper = postcard::from_bytes(&buffer)?;
    Ok(P::try_new_from_list(params.list)?)
}

pub fn run_project<P, F>(generate_function: F) -> Result<(), Box<dyn Error>>
where
    P: PlotteryParams,
    F: Fn(P) -> Layer,
{
    let args = Args::parse();
    match args.command {
        RunCommand::Svg { path } => {
            let open_after = path.is_none();
            let path =
                path.map_or_else(|| std::env::temp_dir().join("temp_cli.svg"), PathBuf::from);
            if path.is_dir() {
                return Err(
                    format!("Failed to write .svg, '{}' is a directory", path.display()).into(),
                );
            }
            generate_function(load_params(false)?).write_svg(path.clone())?;
            if open_after {
                open::that(path)?;
            }
        }
        RunCommand::StdOut { piped_params } => {
            let art = generate_function(load_params(piped_params.unwrap_or(false))?);
            io::stdout().lock().write_all(&art.to_binary()?)?;
        }
        RunCommand::Socket {
            path,
            piped_params,
            frame_every,
        } => {
            let params = load_params(piped_params)?;
            let mut stream = UnixStream::connect(&path)?;
            if let Some(every) = frame_every {
                let mut frames = stream.try_clone()?;
                record_frames(every, move |frame| {
                    write_message(&mut frames, MessageKind::Snapshot, &frame)
                        .expect("Failed to send frame")
                });
            }
            let art = generate_function(params);
            write_message(&mut stream, MessageKind::Final, &art)?;
        }
        RunCommand::Dry {} => {
            generate_function(load_params(false)?);
        }
        RunCommand::Params {} => {
            let mut stdout = io::stdout().lock();
            let params_list = ProjectParamsListWrapper::new(P::param_defaults_list());
            let binary = postcard::to_stdvec(&params_list)?;
            stdout.write_all(&binary)?;
        }
        RunCommand::ProtocolVersion {} => println!("{PROTOCOL_VERSION}"),
    }
    Ok(())
}
