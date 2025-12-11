use super::error::KDBError;

use defmt::Format;

#[derive(Clone, Copy, Format)]
pub struct KDBHeader {
    // https://gist.github.com/lgg/e6ccc6e212d18dd2ecd8a8c116fb1e45
    pub flags: u32,
    pub subversion: u32,
    pub master_seed: [u8; 16],
    pub encryption_iv: [u8; 16],
    pub num_groups: u32,
    pub num_entries: u32,
    pub contents_hash: [u8; 32],
    pub transform_seed: [u8; 32],
    pub transform_rounds: u32,
}

// First 4 bytes are the KeePass magic.
pub const HEADER_SIZE: usize = 4 + 4 + 4 + 4 + 16 + 16 + 4 + 4 + 32 + 32 + 4;

pub fn parse_header(data: &[u8]) -> Result<KDBHeader, KDBError> {
    if data.len() < HEADER_SIZE {
        return Err(KDBError::DatabaseIntegrityError);
    }

    // First 4 bytes are the magic; skip them for now and parse fields.
    Ok(KDBHeader {
        flags: read_u32_le(data, 4)?,
        subversion: read_u32_le(data, 8)?,
        master_seed: copy_array::<16>(data, 12)?,
        encryption_iv: copy_array::<16>(data, 28)?,
        num_groups: read_u32_le(data, 44)?,
        num_entries: read_u32_le(data, 48)?,
        contents_hash: copy_array::<32>(data, 52)?,
        transform_seed: copy_array::<32>(data, 84)?,
        transform_rounds: read_u32_le(data, 116)?,
    })
}

fn read_u32_le(data: &[u8], start: usize) -> Result<u32, KDBError> {
    let bytes: [u8; 4] = data
        .get(start..start + 4)
        .ok_or(KDBError::DatabaseIntegrityError)?
        .try_into()
        .map_err(|_| KDBError::DatabaseIntegrityError)?;
    Ok(u32::from_le_bytes(bytes))
}

fn copy_array<const N: usize>(data: &[u8], start: usize) -> Result<[u8; N], KDBError> {
    let slice = data
        .get(start..start + N)
        .ok_or(KDBError::DatabaseIntegrityError)?;
    let mut out = [0u8; N];
    out.copy_from_slice(slice);
    Ok(out)
}
