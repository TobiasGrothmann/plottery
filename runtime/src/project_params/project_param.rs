use serde::{Deserialize, Serialize};

use super::project_param_value::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl PartialEq for ProjectParam {
    fn eq(&self, other: &Self) -> bool {
        // section, description and `#[range]` don't decide whether a saved value is kept.
        self.name == other.name && value_schema_equal(&self.value, &other.value)
    }
}

/// Same type, whatever the values and `#[range]`s.
fn value_schema_equal(left: &ProjectParamValue, right: &ProjectParamValue) -> bool {
    use ProjectParamValue::{
        DataEnum, Enum, Float, FloatRanged, Int, IntRanged, Optional, Struct, Vec,
    };
    match (left, right) {
        (Float(_) | FloatRanged { .. }, Float(_) | FloatRanged { .. }) => true,
        (Int(_) | IntRanged { .. }, Int(_) | IntRanged { .. }) => true,
        (Struct(left_struct), Struct(right_struct)) => left_struct.fields == right_struct.fields,
        (Optional(left_optional), Optional(right_optional)) => {
            value_schema_equal(&left_optional.value, &right_optional.value)
        }
        (Vec(left_vec), Vec(right_vec)) => {
            value_schema_equal(&left_vec.item_prototype, &right_vec.item_prototype)
        }
        (Enum(left_enum), Enum(right_enum)) => left_enum.variants == right_enum.variants,
        (DataEnum(left_enum), DataEnum(right_enum)) => left_enum.variants == right_enum.variants,
        _ => std::mem::discriminant(left) == std::mem::discriminant(right),
    }
}

impl ProjectParam {
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
