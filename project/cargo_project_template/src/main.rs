use clap::{Parser, Subcommand};
use std::error::Error;
use std::io::{self, Write};
use std::path::PathBuf;

mod generate;
use generate::*;

#[derive(Debug, Clone, Subcommand)]
enum RunCommand {
    Svg {
        path: Option<String>,
        scale: Option<f32>,
    },
    StdOut {},
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    match args.command {
        RunCommand::Svg { path, scale } => {
            let scale = scale.unwrap_or(10.0);

            match path {
                Some(path_string) => {
                    let path = PathBuf::from(&path_string);
                    if path.is_dir() {
                        panic!(
                            "Failed to write SVG. Path '{}' is a directory.",
                            path_string
                        );
                    }
                    let art = generate();
                    art.write_svg(path, scale)?;
                }
                None => {
                    let path = std::env::temp_dir().join("{{project-name}}.svg");
                    let art = generate();
                    art.write_svg(path.clone(), scale)?;

                    open::that_in_background(path).join().unwrap().unwrap();
                }
            }
        }
        RunCommand::StdOut {} => {
            let mut stdout = io::stdout().lock();
            let art = generate();
            let binary = art.to_binary().unwrap();
            stdout.write_all(&binary)?;
        }
    }
    Ok(())
}
