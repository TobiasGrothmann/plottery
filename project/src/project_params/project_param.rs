use serde::{Deserialize, Serialize};

use super::project_param_value::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParam {
    pub name: String,
    pub value: ProjectParamValue,
}

impl PartialEq for ProjectParam {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && value_schema_equal(&self.value, &other.value)
    }
}

fn value_schema_equal(left: &ProjectParamValue, right: &ProjectParamValue) -> bool {
    match (left, right) {
        (
            ProjectParamValue::FloatRanged { val: _, min, max },
            ProjectParamValue::FloatRanged {
                val: _,
                min: min_other,
                max: max_other,
            },
        ) => min == min_other && max == max_other,
        (
            ProjectParamValue::IntRanged { val: _, min, max },
            ProjectParamValue::IntRanged {
                val: _,
                min: min_other,
                max: max_other,
            },
        ) => min == min_other && max == max_other,
        (ProjectParamValue::Struct(left_struct), ProjectParamValue::Struct(right_struct)) => {
            left_struct.fields.len() == right_struct.fields.len()
                && left_struct
                    .fields
                    .iter()
                    .zip(right_struct.fields.iter())
                    .all(|(left_field, right_field)| left_field == right_field)
        }
        (
            ProjectParamValue::Optional(left_optional),
            ProjectParamValue::Optional(right_optional),
        ) => value_schema_equal(&left_optional.value, &right_optional.value),
        (ProjectParamValue::Vec(left_vec), ProjectParamValue::Vec(right_vec)) => {
            value_schema_equal(&left_vec.item_prototype, &right_vec.item_prototype)
        }
        _ => left.type_name() == right.type_name(),
    }
}

impl ProjectParam {
    pub fn new(name: &str, value: ProjectParamValue) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }

    pub fn formatted_name(&self) -> String {
        self.name.replace('_', " ").to_lowercase()
    }
}
