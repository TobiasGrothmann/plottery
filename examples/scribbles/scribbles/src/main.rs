mod generate;

use std::error::Error;

use generate::*;
use plottery_project::run_project;

fn main() -> Result<(), Box<dyn Error>> {
    run_project(generate)
}
