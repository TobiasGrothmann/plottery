use serde::{Deserialize, Serialize};

/// Value of a `#[derive(PlotteryEnum)]` field: the selected variant and all variant names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectParamEnum {
    pub selected: String,
    pub variants: Vec<String>,
}

impl ProjectParamEnum {
    pub fn new(selected: &str, variants: &[&str]) -> Self {
        Self {
            selected: selected.to_string(),
            variants: variants.iter().map(|v| v.to_string()).collect(),
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.variants.iter().position(|v| *v == self.selected)
    }
}
