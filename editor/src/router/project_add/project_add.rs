use crate::{
    components::navigation::Navigation, router::project_add::project_create::ProjectCreate,
};
use dioxus::prelude::*;

#[component]
pub fn ProjectAdd() -> Element {
    rsx! {
        style { { include_str!("./project_add.css") } }
        Navigation { page_name: "Create project", body: rsx! {} }

        div { class: "ProjectAdd",
            div { class: "project_add_content",
                ProjectCreate {}
            }
        }
    }
}
