use anyhow::Result;
use plottery_lib::V2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectParamValue {
    Float(f32),
    Int(i32),
    V2(V2),
}

impl ProjectParamValue {
    pub fn get_f32(&self) -> Result<f32> {
        match self {
            ProjectParamValue::Float(val) => Ok(*val),
            _ => Err(anyhow::anyhow!("Value is not a float")),
        }
    }

    #[allow(non_snake_case)]
    pub fn get_V2(&self) -> Result<V2> {
        match self {
            ProjectParamValue::V2(val) => Ok(*val),
            _ => Err(anyhow::anyhow!("Value is not a V2")),
        }
    }
}
