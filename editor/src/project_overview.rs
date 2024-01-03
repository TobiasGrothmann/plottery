use dioxus::prelude::*;
use path_absolutize::Absolutize;
use plottery_cli::Project;

#[derive(PartialEq, Props)]
pub struct ProjectOverviewProps {
    pub project: Project,
}

pub fn ProjectOverview(cx: Scope<ProjectOverviewProps>) -> Element {
    cx.render(rsx! {
        div {
            p { cx.props.project.config.name.clone() }
            p { cx.props.project.dir.absolutize().unwrap().to_string_lossy() }
            p { (if cx.props.project.exists() {"exists"} else {"does not exist"}) }
        }
    })
}
