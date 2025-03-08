use bincode::serialize;
use clap::{Parser, Subcommand};
use plottery_lib::Layer;
use std::error::Error;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use crate::{PlotteryParamsDefinition, ProjectParam, ProjectParamsListWrapper};

#[derive(Debug, Clone, Subcommand)]
enum RunCommand {
    Svg {
        path: Option<String>,
        scale: Option<f32>,
    },
    StdOut {
        #[arg(long, short)]
        piped_params: Option<bool>,
    },
    Dry {},
    Params {},
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
    let params_list_wrapper: ProjectParamsListWrapper = bincode::deserialize(&buffer)?;
    Ok(params_list_wrapper.list)
}

pub fn run_project<P, F>(generate_function: F) -> Result<(), Box<dyn Error>>
where
    P: PlotteryParamsDefinition,
    F: Fn(P) -> Layer,
{
    let args = Args::parse();
    match args.command {
        RunCommand::Svg { path, scale } => {
            let scale = scale.unwrap_or(10.0);

            match path {
                Some(path_string) => {
                    let path = PathBuf::from(&path_string);
                    if path.is_dir() {
                        panic!(
                            "Failed to write .svg - Path '{}' is a directory.",
                            path_string
                        );
                    }
                    let art = generate_function(P::new_with_defaults());
                    art.write_svg(path, scale)?;
                }
                None => {
                    let path = std::env::temp_dir().join("temp_cli.svg");
                    let art = generate_function(P::new_with_defaults());
                    art.write_svg(path.clone(), scale)?;

                    open::that_in_background(path)
                        .join()
                        .expect("Failed to open svg.")
                        .expect("Failed to open svg.");
                }
            }
        }
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
            let binary = art.to_binary().expect("Failed to convert layer to binary.");
            stdout.write_all(&binary)?;
        }
        RunCommand::Dry {} => {
            generate_function(P::new_with_defaults());
        }
        RunCommand::Params {} => {
            let mut stdout = io::stdout().lock();
            let params_list = ProjectParamsListWrapper::new(P::param_defaults_list());
            let binary = serialize(&params_list)?;
            stdout.write_all(&binary)?;
        }
    }
    Ok(())
}
