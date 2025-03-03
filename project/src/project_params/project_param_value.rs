use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProjectParamValue {
    Float(f32),
    FloatRanged { val: f32, min: f32, max: f32 },
    Int(i32),
    IntRanged { val: i32, min: i32, max: i32 },
    Bool(bool),
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
                self.type_name()
            )),
        }
    }
    pub fn set_f32(&mut self, new_val: f32) {
        match self {
            ProjectParamValue::Float(val) => *val = new_val,
            ProjectParamValue::FloatRanged { val, min, max } => *val = new_val.clamp(*min, *max),
            _ => panic!("Failed to set_f32 - Type is '{}'.", self.type_name()),
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
                self.type_name()
            )),
        }
    }
    pub fn set_i32(&mut self, new_val: i32) {
        match self {
            ProjectParamValue::Int(val) => *val = new_val,
            ProjectParamValue::IntRanged { val, min, max } => *val = new_val.clamp(*min, *max),
            _ => panic!("Failed to set_i32 - Type is '{}'.", self.type_name()),
        }
    }

    pub fn get_bool(&self) -> Result<bool> {
        match self {
            ProjectParamValue::Bool(val) => Ok(*val),
            _ => Err(anyhow::anyhow!(
                "Failed to get_bool - Expected value to be of type 'bool' but found {}",
                self.type_name()
            )),
        }
    }
    pub fn set_bool(&mut self, new_val: bool) {
        match self {
            ProjectParamValue::Bool(val) => *val = new_val,
            _ => panic!("Failed to set_bool - Type is '{}'.", self.type_name()),
        }
    }

    pub fn type_name(&self) -> String {
        match self {
            ProjectParamValue::Float(_) => "f32".to_string(),
            ProjectParamValue::FloatRanged { .. } => "f32 (ranged)".to_string(),
            ProjectParamValue::Int(_) => "i32".to_string(),
            ProjectParamValue::IntRanged { .. } => "i32 (ranged)".to_string(),
            ProjectParamValue::Bool(_) => "bool".to_string(),
        }
    }

    pub fn value_as_string(&self) -> String {
        match self {
            ProjectParamValue::Float(val) => val.to_string(),
            ProjectParamValue::FloatRanged { val, .. } => val.to_string(),
            ProjectParamValue::Int(val) => val.to_string(),
            ProjectParamValue::IntRanged { val, .. } => val.to_string(),
            ProjectParamValue::Bool(val) => val.to_string(),
        }
    }
}
