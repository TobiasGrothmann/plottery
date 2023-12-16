// WIP: just for testing things

use project::Project;
use std::path::Path;

fn main() {
    let project_name: &str = "test_proj";
    let project = Project::new(
        Path::new(&"/Users/admin/Downloads".to_string()).to_path_buf(),
        project_name.into(),
    );
    project.generate_to_disk().unwrap();
}
