use super::error::KDBError;

use defmt::Format;

pub const KDB_SIGNATURE1: u32 = 0x9AA2D903;
pub const KDB_SIGNATURE2: u32 = 0xB54BFB65;

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

/// Fixed KDB1 header size, including the two 4-byte signatures.
pub const HEADER_SIZE: usize = 4 + 4 + 4 + 4 + 16 + 16 + 4 + 4 + 32 + 32 + 4;

impl KDBHeader {
    pub fn new_from_header(data: &[u8]) -> Result<KDBHeader, KDBError> {
        if data.len() < HEADER_SIZE {
            return Err(KDBError::DatabaseIntegrityError);
        }

        let signature1 = read_u32_le(data, 0)?;
        let signature2 = read_u32_le(data, 4)?;
        if signature1 != KDB_SIGNATURE1 || signature2 != KDB_SIGNATURE2 {
            return Err(KDBError::DatabaseIntegrityError);
        }

        // Skip the two 4-byte signatures and parse the remaining fields.
        Ok(KDBHeader {
            flags: read_u32_le(data, 8)?,
            subversion: read_u32_le(data, 12)?,
            master_seed: copy_array::<16>(data, 16)?,
            encryption_iv: copy_array::<16>(data, 32)?,
            num_groups: read_u32_le(data, 48)?,
            num_entries: read_u32_le(data, 52)?,
            contents_hash: copy_array::<32>(data, 56)?,
            transform_seed: copy_array::<32>(data, 88)?,
            transform_rounds: read_u32_le(data, 120)?,
        })
    }
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
