use crate::{
    components::navigation::Navigation,
    router::add_project::{ProjectCreate, ProjectImport},
};
use dioxus::prelude::*;

enum ProjectAddTypes {
    Create,
    Import,
}

#[component]
pub fn ProjectAdd() -> Element {
    let mut project_add_type = use_signal(|| ProjectAddTypes::Create);

    let create_class = match *project_add_type.read() {
        ProjectAddTypes::Create => "segmented_button_item_selected",
        _ => "",
    };
    let import_class = match *project_add_type.read() {
        ProjectAddTypes::Import => "segmented_button_item_selected",
        _ => "",
    };

    rsx! {
        style { { include_str!("./project_add.css") } }
        Navigation { page_name: "Add project" }

        div { class: "ProjectAdd",
            div { class: "segmented_button",
                button { class: "segmented_button_item {create_class}",
                    onclick: move |_event| {
                        project_add_type.set(ProjectAddTypes::Create)
                    },
                    "Create"
                }
                button { class: "segmented_button_item {import_class}",
                    onclick: move |_event| {
                        project_add_type.set(ProjectAddTypes::Import)
                    },
                    "Import",
                }
            }
            div { class: "project_add_content",
                if let ProjectAddTypes::Create = *project_add_type.read() {
                    ProjectCreate {}
                } else if let ProjectAddTypes::Import = *project_add_type.read(){
                    ProjectImport {}
                }
            }
        }
    }
}
