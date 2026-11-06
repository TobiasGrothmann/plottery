use std::path::PathBuf;

use path_absolutize::Absolutize;
use plottery_project::Project;

use crate::routes::Route;

/// The project's config path split into route segments.
pub fn project_path_segments(project: &Project) -> Vec<String> {
    let config_path = project.get_project_config_path();
    let path_str = config_path
        .absolutize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| config_path.to_string_lossy().to_string());
    path_str
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn editor_route(project: &Project) -> Route {
    Route::Editor {
        project_path: project_path_segments(project),
    }
}

/// Loads the project behind a route's path segments (inverse of [`project_path_segments`]).
pub fn project_from_segments(segments: &[String]) -> anyhow::Result<Project> {
    Project::load_from_file(PathBuf::from("/").join(segments.join("/")))
}
