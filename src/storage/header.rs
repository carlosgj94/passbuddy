use defmt::Format;

// This offset is used so the storage writes don't overlap with the bootloader and flash.
const STORAGE_OFFSET: u32 = 0x200000;
pub const STORAGE_MAGIC: [u8; 4] = *b"PBDY";
pub const STORAGE_LAYOUT_VERSION: u16 = 1;
pub(crate) const LAYOUT_HEADER_SIZE: usize = 8;

/// Small header to sit ahead of the descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct LayoutHeader {
    pub magic: [u8; 4],
    pub layout_version: u16,
    pub region_count: u8,
}

/// Return the expected header. For now this is a constant; later it can read from flash.
pub fn get_header() -> LayoutHeader {
    LayoutHeader {
        magic: STORAGE_MAGIC,
        layout_version: STORAGE_LAYOUT_VERSION,
        region_count: 4, // ProjectConfig, UserConfig, KeePassDb, Scratch
    }
}

pub(crate) const fn storage_magic_offset() -> u32 {
    STORAGE_OFFSET
}

/// Offset where the layout header begins (immediately after the magic marker).
///
/// This is a pure computation and performs no flash I/O.
pub(crate) const fn get_user_storage_offset() -> u32 {
    STORAGE_OFFSET + 4
}

impl LayoutHeader {
    pub(crate) fn new_from_bytes(bytes: &[u8; LAYOUT_HEADER_SIZE]) -> Self {
        LayoutHeader {
            magic: bytes[0..4].try_into().unwrap(),
            layout_version: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            region_count: bytes[6],
        }
    }
    pub(crate) fn get_bytes(&self) -> [u8; LAYOUT_HEADER_SIZE] {
        let mut bytes = [0u8; LAYOUT_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.layout_version.to_le_bytes());
        bytes[6] = self.region_count;
        bytes
    }
}
