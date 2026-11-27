use anyhow::{bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Puts the header of a Plottery file in front of `payload`: 4 magic bytes and the format version as little endian `u16`.
pub fn with_file_header(magic: &[u8; 4], version: u16, payload: &[u8]) -> Vec<u8> {
    [magic.as_slice(), &version.to_le_bytes(), payload].concat()
}

/// The format version and the payload of a file, `None` if it does not start with `magic` and a version.
pub fn split_file_header<'a>(magic: &[u8; 4], bytes: &'a [u8]) -> Option<(u16, &'a [u8])> {
    let rest = bytes.strip_prefix(magic)?;
    let (version, payload) = rest.split_first_chunk::<2>()?;
    Some((u16::from_le_bytes(*version), payload))
}

/// `value` as postcard behind a file header, see [`read_versioned`].
pub fn write_versioned<T: Serialize>(magic: &[u8; 4], version: u16, value: &T) -> Result<Vec<u8>> {
    Ok(with_file_header(
        magic,
        version,
        &postcard::to_stdvec(value)?,
    ))
}

/// Reads what [`write_versioned`] wrote; the error names `kind` and says if the file is foreign, older, newer or damaged.
pub fn read_versioned<T: DeserializeOwned>(
    magic: &[u8; 4],
    version: u16,
    bytes: &[u8],
    kind: &str,
) -> Result<T> {
    Ok(read_versioned_since(magic, version, version, bytes, kind)?.1)
}

/// [`read_versioned`] for a format whose outer layout is the same from `oldest` to `version`, with
/// the version found: the caller migrates what changed inside.
pub fn read_versioned_since<T: DeserializeOwned>(
    magic: &[u8; 4],
    oldest: u16,
    version: u16,
    bytes: &[u8],
    kind: &str,
) -> Result<(u16, T)> {
    let Some((found, payload)) = split_file_header(magic, bytes) else {
        bail!("not a {kind} file of this Plottery version");
    };
    if found < oldest {
        bail!("the {kind} file has the old format version {found}, which this Plottery can't read anymore");
    }
    if found > version {
        bail!("the {kind} file has format version {found}, this Plottery reads up to {version}. Update Plottery.");
    }
    let value =
        postcard::from_bytes(payload).with_context(|| format!("the {kind} file is damaged"))?;
    Ok((found, value))
}
