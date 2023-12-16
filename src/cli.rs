use std::path::PathBuf;

use clap::{Parser, Subcommand};
use plottery_lib::Project;

#[derive(Debug, Subcommand)]
enum Commands {
    New { path: String, name: String },
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
    }
}
