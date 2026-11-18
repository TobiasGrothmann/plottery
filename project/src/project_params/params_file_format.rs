//! The `.plotp` format: a file header, the number of params and one record per top-level param.
//! Each value is its own length-prefixed block, so a value this version can't decode costs only
//! that param.

use anyhow::{bail, Context, Result};
use bincode::Options;
use plottery_lib::{split_file_header, with_file_header};
use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;
use super::project_param_value::ProjectParamValue;
use super::project_params_list_wrapper::ProjectParamsListWrapper;

const MAGIC: [u8; 4] = *b"PLTP";
const VERSION: u16 = 1;

#[derive(Serialize, Deserialize)]
struct ParamRecord {
    name: String,
    /// Index of the value's variant in the append-only [`ProjectParamValue`].
    type_tag: u32,
    /// Bincode of the variant's content.
    value: Vec<u8>,
    section: Option<String>,
    description: Option<String>,
}

impl ParamRecord {
    fn new(param: &ProjectParam) -> Result<Self> {
        // bincode starts an enum with its variant index as u32
        let binary = bincode::serialize(&param.value)?;
        let (type_tag, value) = binary
            .split_first_chunk::<4>()
            .context("param value without a type")?;
        Ok(Self {
            name: param.name.clone(),
            type_tag: u32::from_le_bytes(*type_tag),
            value: value.to_vec(),
            section: param.section.clone(),
            description: param.description.clone(),
        })
    }

    /// Fails for a type or a value layout this version does not know.
    fn value(&self) -> Result<ProjectParamValue> {
        let binary = [self.type_tag.to_le_bytes().as_slice(), &self.value].concat();
        Ok(exact().deserialize(&binary)?)
    }
}

/// Bincode as `bincode::deserialize` reads it, but leftover bytes are an error.
fn exact() -> impl Options {
    bincode::options().with_fixint_encoding()
}

impl ProjectParamsListWrapper {
    pub fn to_file_bytes(&self) -> Result<Vec<u8>> {
        let records = self
            .list
            .iter()
            .map(ParamRecord::new)
            .collect::<Result<Vec<_>>>()?;
        Ok(with_file_header(
            &MAGIC,
            VERSION,
            &bincode::serialize(&records)?,
        ))
    }

    /// The saved params, and the names of those left out because their value could not be decoded.
    pub fn from_file_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)> {
        let records: Vec<ParamRecord> = match split_file_header(&MAGIC, bytes) {
            // written before files had a header: the bincode of the whole list
            None => {
                let params = exact()
                    .deserialize(bytes)
                    .context("not a params file of this or an older Plottery version")?;
                return Ok((params, Vec::new()));
            }
            Some((VERSION, payload)) => exact()
                .deserialize(payload)
                .context("the params file is damaged")?,
            Some((version, _)) => bail!(
                "the params file has format version {version}, this Plottery reads up to {VERSION}. Update Plottery."
            ),
        };

        let mut skipped = Vec::new();
        let list = records
            .into_iter()
            .filter_map(|record| match record.value() {
                Ok(value) => Some(ProjectParam {
                    name: record.name,
                    value,
                    section: record.section,
                    description: record.description,
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
