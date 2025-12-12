use defmt::Format;

pub(crate) const REGION_DESCRIPTOR_SIZE: usize = 20;

/// Regions we plan to keep in flash. Add more as the layout evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
#[repr(u8)]
pub enum DataRegion {
    ProjectConfig,
    UserConfig,
    KeePassDb,
    Scratch,
}

/// Describes where a region lives in flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct RegionDescriptor {
    pub kind: DataRegion,
    pub offset: u32,
    /// Total bytes reserved for this region in flash.
    pub capacity: u32,
    /// Bytes currently used (header + ciphertext). 0 means empty/uninitialized.
    pub used_len: u32,
    /// TODO: Populate/validate CRC32 for region contents.
    pub crc32: u32,
}

impl RegionDescriptor {
    pub const fn empty() -> Self {
        Self {
            kind: DataRegion::Scratch,
            offset: 0,
            capacity: 0,
            used_len: 0,
            crc32: 0,
        }
    }
    pub const fn empty_with_kind(kind: DataRegion) -> Self {
        Self {
            kind,
            offset: 0,
            capacity: 0,
            used_len: 0,
            crc32: 0,
        }
    }

    pub(crate) fn new_from_bytes(bytes: &[u8; REGION_DESCRIPTOR_SIZE]) -> Self {
        let kind = match bytes[0] {
            0 => DataRegion::ProjectConfig,
            1 => DataRegion::UserConfig,
            2 => DataRegion::KeePassDb,
            3 => DataRegion::Scratch,
            _ => panic!("Invalid region kind"),
        };
        RegionDescriptor {
            kind: kind,
            offset: u32::from_le_bytes(bytes[1..5].try_into().unwrap()),
            capacity: u32::from_le_bytes(bytes[5..9].try_into().unwrap()),
            used_len: u32::from_le_bytes(bytes[9..13].try_into().unwrap()),
            crc32: u32::from_le_bytes(bytes[13..17].try_into().unwrap()),
        }
    }
    pub(crate) fn to_bytes(&self) -> [u8; REGION_DESCRIPTOR_SIZE] {
        let mut bytes = [0u8; REGION_DESCRIPTOR_SIZE];
        bytes[0] = self.kind as u8;
        bytes[1..5].copy_from_slice(&self.offset.to_le_bytes());
        bytes[5..9].copy_from_slice(&self.capacity.to_le_bytes());
        bytes[9..13].copy_from_slice(&self.used_len.to_le_bytes());
        bytes[13..17].copy_from_slice(&self.crc32.to_le_bytes());
        bytes
    }
}
