use clap::{Parser, Subcommand};
use plottery_lib::{record_frames, Layer};
use std::error::Error;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::project_param::ProjectParam;
use crate::project_params_definition::PlotteryParams;
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

fn read_params_from_stdin() -> Result<Vec<ProjectParam>, Box<dyn Error>> {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer)?;
    let params_list_wrapper: ProjectParamsListWrapper = postcard::from_bytes(&buffer)?;
    Ok(params_list_wrapper.list)
}

pub fn run_project<P, F>(generate_function: F) -> Result<(), Box<dyn Error>>
where
    P: PlotteryParams,
    F: Fn(P) -> Layer,
{
    let args = Args::parse();
    match args.command {
        RunCommand::Svg { path } => match path {
            Some(path_string) => {
                let path = PathBuf::from(&path_string);
                if path.is_dir() {
                    panic!(
                        "Failed to write .svg - Path '{}' is a directory.",
                        path_string
                    );
                }
                let art = generate_function(P::new_with_defaults());
                art.write_svg(path)?;
            }
            None => {
                let path = std::env::temp_dir().join("temp_cli.svg");
                let art = generate_function(P::new_with_defaults());
                art.write_svg(path.clone())?;

                open::that_in_background(path)
                    .join()
                    .expect("Failed to join background process for opening SVG")
                    .expect("Failed to open SVG file in default viewer");
            }
        },
        RunCommand::StdOut { piped_params } => {
            let wait_for_stdin = piped_params.unwrap_or(false);

            let params = if wait_for_stdin {
                let list = read_params_from_stdin()?;
                P::new_from_list(list)
            } else {
                P::new_with_defaults()
            };

            let mut stdout = io::stdout().lock();
            let art = generate_function(params);
            let binary = art
                .to_binary()
                .expect("Failed to convert layer to binary format");
            stdout.write_all(&binary)?;
        }
        RunCommand::Socket {
            path,
            piped_params,
            frame_every,
        } => {
            let params = if piped_params {
                let list = read_params_from_stdin()?;
                P::new_from_list(list)
            } else {
                P::new_with_defaults()
            };

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
            generate_function(P::new_with_defaults());
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
