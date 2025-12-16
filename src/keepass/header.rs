use super::error::KDBError;

use defmt::Format;

pub const KDB_SIGNATURE1: u32 = 0x9AA2D903;
pub const KDB_SIGNATURE2: u32 = 0xB54BFB65;

#[derive(Clone, Copy, Format, Debug)]
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

/// Fixed KDB1 header size, without including the two 4-byte signatures.
pub const HEADER_SIZE: usize = 4 + 4 + 16 + 16 + 4 + 4 + 32 + 32 + 4;

impl KDBHeader {
    pub fn empty() -> KDBHeader {
        KDBHeader {
            flags: 0,
            subversion: 0,
            master_seed: [0; 16],
            encryption_iv: [0; 16],
            num_groups: 0,
            num_entries: 0,
            contents_hash: [0; 32],
            transform_seed: [0; 32],
            transform_rounds: 0,
        }
    }

    pub fn new_from_bytes(data: &[u8]) -> Result<KDBHeader, KDBError> {
        if data.len() < HEADER_SIZE {
            return Err(KDBError::DatabaseIntegrityError);
        }

        // Skip the two 4-byte signatures and parse the remaining fields.
        Ok(KDBHeader {
            flags: read_u32_le(data, 0)?,
            subversion: read_u32_le(data, 4)?,
            master_seed: copy_array::<16>(data, 8)?,
            encryption_iv: copy_array::<16>(data, 24)?,
            num_groups: read_u32_le(data, 40)?,
            num_entries: read_u32_le(data, 44)?,
            contents_hash: copy_array::<32>(data, 48)?,
            transform_seed: copy_array::<32>(data, 80)?,
            transform_rounds: read_u32_le(data, 112)?,
        })
    }
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut bytes = [0u8; HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.flags.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.subversion.to_le_bytes());
        bytes[8..24].copy_from_slice(&self.master_seed);
        bytes[24..40].copy_from_slice(&self.encryption_iv);
        bytes[40..44].copy_from_slice(&self.num_groups.to_le_bytes());
        bytes[44..48].copy_from_slice(&self.num_entries.to_le_bytes());
        bytes[48..80].copy_from_slice(&self.contents_hash);
        bytes[80..112].copy_from_slice(&self.transform_seed);
        bytes[112..116].copy_from_slice(&self.transform_rounds.to_le_bytes());
        bytes
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
