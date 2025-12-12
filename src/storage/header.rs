use defmt::Format;
use embedded_storage::{ReadStorage, Storage};
use esp_storage::FlashStorage;

// This offset is used so the storage writes don't overlap with the bootloader and flash.
const STORAGE_OFFSET: u32 = 0x200000;
pub const STORAGE_MAGIC: [u8; 4] = *b"PBDY";
pub const STORAGE_LAYOUT_VERSION: u16 = 1;

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

/// Returns the esp32s3 offset + the internal offset + the magic number offset
/// It also checks the magic number is there or writes it
pub(crate) fn get_user_storage_offset(storage: &mut FlashStorage) -> u32 {
    // 1. We read what's after the esp_offset
    let mut magic_buffer = [0u8; 4];
    storage
        .read(STORAGE_OFFSET, &mut magic_buffer)
        .expect("read failed");

    // 2. We check the magic number
    if magic_buffer != STORAGE_MAGIC {
        storage
            .write(STORAGE_OFFSET, &STORAGE_MAGIC)
            .expect("write failed");
    }

    STORAGE_OFFSET + 4
}

impl LayoutHeader {
    pub(crate) fn new_from_bytes(bytes: &[u8; core::mem::size_of::<LayoutHeader>()]) -> Self {
        LayoutHeader {
            magic: bytes[0..4].try_into().unwrap(),
            layout_version: u16::from_le_bytes(bytes[4..6].try_into().unwrap()),
            region_count: bytes[6],
        }
    }
    pub(crate) fn get_bytes(&self) -> [u8; core::mem::size_of::<LayoutHeader>()] {
        let mut bytes = [0u8; core::mem::size_of::<LayoutHeader>()];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.layout_version.to_le_bytes());
        bytes[6] = self.region_count;
        bytes
    }
}
