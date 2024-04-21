use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectParamValue {
    Float(f32),
    FloatRanged { val: f32, min: f32, max: f32 },
    Int(i32),
    IntRanged { val: i32, min: i32, max: i32 },
}

impl ProjectParamValue {
    pub fn get_f32(&self) -> Result<f32> {
        match self {
            ProjectParamValue::Float(val) => Ok(*val),
            ProjectParamValue::FloatRanged {
                val,
                min: _,
                max: _,
            } => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Failed to get_f32 - Expected value to be of type 'float' but found {}",
                self.get_type_name()
            )),
        }
    }

    pub fn get_i32(&self) -> Result<i32> {
        match self {
            ProjectParamValue::Int(val) => Ok(*val),
            ProjectParamValue::IntRanged {
                val,
                min: _,
                max: _,
            } => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Failed to get_i32 - Expected value to be of type 'int' but found {}",
                self.get_type_name()
            )),
        }
    }

    pub fn get_type_name(&self) -> String {
        match self {
            ProjectParamValue::Float(_) => "f32".to_string(),
            ProjectParamValue::FloatRanged { .. } => "f32 (ranged)".to_string(),
            ProjectParamValue::Int(_) => "i32".to_string(),
            ProjectParamValue::IntRanged { .. } => "i32 (ranged)".to_string(),
        }
    }
}
