use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;

/// Value of a `#[derive(PlotteryEnum)]` field whose variants have fields: the selected variant
/// and the field values of every variant, so switching back keeps them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamDataEnum {
    pub selected: String,
    pub variants: Vec<ProjectParamVariant>,
}

/// One variant of a [`ProjectParamDataEnum`]. Unit variants have no fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamVariant {
    pub name: String,
    pub fields: Vec<ProjectParam>,
}

impl ProjectParamVariant {
    pub fn new(name: &str, fields: Vec<ProjectParam>) -> Self {
        Self {
            name: name.to_string(),
            fields,
        }
    }
}

impl ProjectParamDataEnum {
    pub fn new(selected: &str, variants: Vec<ProjectParamVariant>) -> Self {
        Self {
            selected: selected.to_string(),
            variants,
        }
    }

    pub fn variant_names(&self) -> Vec<String> {
        self.variants.iter().map(|v| v.name.clone()).collect()
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.variants.iter().position(|v| v.name == self.selected)
    }

    pub fn variant(&self, name: &str) -> Option<&ProjectParamVariant> {
        self.variants.iter().find(|v| v.name == name)
    }

    pub fn variant_mut(&mut self, name: &str) -> Option<&mut ProjectParamVariant> {
        self.variants.iter_mut().find(|v| v.name == name)
    }

    /// Fields of the selected variant, empty for a unit variant.
    pub fn selected_fields(&self) -> &[ProjectParam] {
        self.variant(&self.selected)
            .map_or(&[], |variant| variant.fields.as_slice())
    }

    pub fn selected_fields_mut(&mut self) -> &mut [ProjectParam] {
        let selected = self.selected.clone();
        self.variant_mut(&selected)
            .map_or(&mut [], |variant| variant.fields.as_mut_slice())
    }

    /// The selected variant's fields by name. Used by the generated `from_param_value`.
    pub fn selected_fields_map(&self) -> HashMap<String, ProjectParam> {
        self.selected_fields()
            .iter()
            .map(|field| (field.name.clone(), field.clone()))
            .collect()
    }

    /// Selects `name` and gives it `fields`, the other variants keep theirs.
    pub fn with_selected(mut self, name: &str, fields: Vec<ProjectParam>) -> Self {
        if let Some(variant) = self.variant_mut(name) {
            variant.fields = fields;
        }
        self.selected = name.to_string();
        self
    }
}
