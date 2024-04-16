use anyhow::Result;
use plottery_lib::V2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectParamValue {
    Float(f32),
    // RangedFloat { val: f32, min: f32, max: f32 },
    Int(i32),
    // RangedInt { val: i32, min: i32, max: i32 },
    V2(V2),
    // String(String),
    // Bool(bool),
    // Enum { val: String, options: Vec<String> },
}

impl ProjectParamValue {
    pub fn get_float(&self) -> Result<f32> {
        match self {
            ProjectParamValue::Float(val) => Ok(*val),
            _ => Err(anyhow::anyhow!("Value is not a float")),
        }
    }

    // pub fn get_ranged_float(&self) -> Result<f32> {
    //     match self {
    //         ProjectParamValue::RangedFloat { val, min, max } => Ok(val.clamp(*min, *max)),
    //         _ => Err(anyhow::anyhow!("Value is not a ranged float")),
    //     }
    // }

    pub fn get_int(&self) -> Result<i32> {
        match self {
            ProjectParamValue::Int(val) => Ok(*val),
            _ => Err(anyhow::anyhow!("Value is not an int")),
        }
    }

    // pub fn get_ranged_int(&self) -> Result<i32> {
    //     match self {
    //         ProjectParamValue::RangedInt { val, min, max } => Ok((*val).clamp(*min, *max)),
    //         _ => Err(anyhow::anyhow!("Value is not a ranged int")),
    //     }
    // }

    pub fn get_v2(&self) -> Result<V2> {
        match self {
            ProjectParamValue::V2(val) => Ok(*val),
            _ => Err(anyhow::anyhow!("Value is not a V2")),
        }
    }

    // pub fn get_string(&self) -> Result<String> {
    //     match self {
    //         ProjectParamValue::String(val) => Ok(val.clone()),
    //         _ => Err(anyhow::anyhow!("Value is not a string")),
    //     }
    // }

    // pub fn get_bool(&self) -> Result<bool> {
    //     match self {
    //         ProjectParamValue::Bool(val) => Ok(*val),
    //         _ => Err(anyhow::anyhow!("Value is not a bool")),
    //     }
    // }

    // pub fn get_enum(&self) -> Result<String> {
    //     match self {
    //         ProjectParamValue::Enum { val, .. } => Ok(val.clone()),
    //         _ => Err(anyhow::anyhow!("Value is not an enum")),
    //     }
    // }
}
