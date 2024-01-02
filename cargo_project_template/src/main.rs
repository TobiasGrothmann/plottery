use cargo_project_template::generate;

use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    // generate art
    let art = generate();

    // save to svg
    let path = PathBuf::from("tmp.svg");
    art.write_svg(&path, 10.0).unwrap();

    // open svg
    opener::open(path)?;
    Ok(())
}
