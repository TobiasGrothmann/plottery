//! The `.plotp` format: a file header, the number of params and one record per top-level param.
//! Each value is its own length-prefixed block, so a value this version can't decode costs only
//! that param.

use anyhow::{bail, Context, Result};
use plottery_lib::{split_file_header, with_file_header};
use serde::{Deserialize, Serialize};

use super::project_param::ProjectParam;
use super::project_param_value::ProjectParamValue;
use super::project_params_list_wrapper::ProjectParamsListWrapper;

const MAGIC: [u8; 4] = *b"PLTP";
const VERSION: u16 = 3;

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

    /// Fails for a type or a value layout this version does not know.
    fn value(&self) -> Result<ProjectParamValue> {
        let binary = [postcard::to_stdvec(&self.type_tag)?, self.value.clone()].concat();
        let (value, rest) = postcard::take_from_bytes(&binary)?;
        if !rest.is_empty() {
            bail!("param value with leftover bytes");
        }
        Ok(value)
    }
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
            &postcard::to_stdvec(&records)?,
        ))
    }

    /// The saved params, and the names of those left out because their value could not be decoded.
    pub fn from_file_bytes(bytes: &[u8]) -> Result<(Self, Vec<String>)> {
        let records: Vec<ParamRecord> = match split_file_header(&MAGIC, bytes) {
            None => bail!("not a params file of this Plottery version"),
            Some((VERSION, payload)) => {
                postcard::from_bytes(payload).context("the params file is damaged")?
            }
            Some((version, _)) if version < VERSION => bail!(
                "the params file has the old format version {version}, which this Plottery can't read anymore."
            ),
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
