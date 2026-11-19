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
