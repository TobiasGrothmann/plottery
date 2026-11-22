use serde::{Deserialize, Serialize};

use super::project_param_value::ProjectParamValue;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectParam {
    pub name: String,
    pub value: ProjectParamValue,
    /// Editor section from `#[section("...")]`, carried forward to the following fields.
    pub section: Option<String>,
    /// From the field's `///` doc comment.
    pub description: Option<String>,
    /// From `#[no_pick]`: no picking on the preview. Not saved, it travels beside the list, see
    /// `ProjectParamsListWrapper`.
    #[serde(skip)]
    pub no_pick: bool,
}

impl ProjectParam {
    /// Same name and type, whatever the value, `#[range]`, section and description.
    pub fn same_schema(&self, other: &Self) -> bool {
        self.name == other.name && self.value.same_schema(&other.value)
    }

    pub fn new(name: &str, value: ProjectParamValue) -> Self {
        Self {
            name: name.to_string(),
            value,
            section: None,
            description: None,
            no_pick: false,
        }
    }

    pub fn with_no_pick(self) -> Self {
        Self {
            no_pick: true,
            ..self
        }
    }

    pub fn with_section(self, section: &str) -> Self {
        Self {
            section: Some(section.to_string()),
            ..self
        }
    }

    pub fn with_description(self, description: &str) -> Self {
        Self {
            description: Some(description.to_string()),
            ..self
        }
    }

    pub fn formatted_name(&self) -> String {
        self.name.replace('_', " ").to_lowercase()
    }
}
