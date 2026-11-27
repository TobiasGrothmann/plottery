//! The `.plotp` format: a file header, the number of params and one record per top-level param.
//! Each value is its own length-prefixed block, so a value this version can't decode costs only
//! that param.
//!
//! Older versions with the same records are migrated on reading, see `params_file_v2.rs`. A new
//! version that changes how values are stored freezes the current value types into a
//! `params_file_v3.rs` the same way and decodes version 3 records with them.

use anyhow::{bail, Result};
use plottery_lib::{read_versioned_since, write_versioned};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::params_file_v2::ValueV2;
use super::project_param::ProjectParam;
use super::project_param_value::ProjectParamValue;
use super::project_params_list_wrapper::ProjectParamsListWrapper;

const MAGIC: [u8; 4] = *b"PLTP";
const VERSION: u16 = 3;
/// Oldest version that is migrated. Version 1 had other records.
const OLDEST: u16 = 2;

#[derive(Serialize, Deserialize)]
struct ParamRecord {
    name: String,
    /// Index of the value's variant in the append-only [`ProjectParamValue`].
    type_tag: u32,
    /// Postcard of the variant's content.
    value: Vec<u8>,
    section: Option<String>,
    description: Option<String>,
}

impl ParamRecord {
    fn new(param: &ProjectParam) -> Result<Self> {
        // postcard starts an enum with its variant index as a varint u32
        let binary = postcard::to_stdvec(&param.value)?;
        let (type_tag, value) = postcard::take_from_bytes::<u32>(&binary)?;
        Ok(Self {
            name: param.name.clone(),
            type_tag,
            value: value.to_vec(),
            section: param.section.clone(),
            description: param.description.clone(),
        })
    }

    /// The value of a record of format `version`. Fails for a type or a value layout it does
    /// not know.
    fn value(&self, version: u16) -> Result<ProjectParamValue> {
        let binary = [postcard::to_stdvec(&self.type_tag)?, self.value.clone()].concat();
        match version {
            2 => Ok(decode::<ValueV2>(&binary)?.migrate()),
            _ => decode(&binary),
        }
    }
}

fn decode<T: DeserializeOwned>(binary: &[u8]) -> Result<T> {
    let (value, rest) = postcard::take_from_bytes(binary)?;
    if !rest.is_empty() {
        bail!("param value with leftover bytes");
    }
    Ok(value)
}

impl ProjectParamsListWrapper {
    pub fn to_file_bytes(&self) -> Result<Vec<u8>> {
        let records = self
            .list
            .iter()
            .map(ParamRecord::new)
            .collect::<Result<Vec<_>>>()?;
        write_versioned(&MAGIC, VERSION, &records)
    }

    /// The saved params, and the names of those left out because their value could not be decoded.
    pub fn from_file_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)> {
        let (version, records): (u16, Vec<ParamRecord>) =
            read_versioned_since(&MAGIC, OLDEST, VERSION, bytes, "params")?;

        let mut skipped = Vec::new();
        let list = records
            .into_iter()
            .filter_map(|record| match record.value(version) {
                Ok(value) => Some(ProjectParam {
                    name: record.name,
                    value,
                    section: record.section,
                    description: record.description,
                    // comes back from the project's params when they are merged in
                    no_pick: false,
                }),
                Err(_) => {
                    skipped.push(record.name);
                    None
                }
            })
            .collect();
        Ok((Self::new(list), skipped))
    }
}
