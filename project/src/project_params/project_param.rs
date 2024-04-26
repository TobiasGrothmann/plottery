use serde::{Deserialize, Serialize};

use crate::ProjectParamValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectParam {
    pub name: String,
    pub value: ProjectParamValue,
}

impl PartialEq for ProjectParam {
    fn eq(&self, other: &Self) -> bool {
        let name_equal = self.name == other.name;
        let type_equal = self.value.type_name() == other.value.type_name();

        let range_equal = match (&self.value, &other.value) {
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
            _ => true,
        };

        name_equal && type_equal && range_equal
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
