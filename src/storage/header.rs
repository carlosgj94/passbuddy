use defmt::Format;
use embedded_storage::Storage;
use embedded_storage::nor_flash::ReadNorFlash;
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
pub(crate) fn get_offset_user_storage(storage: &mut FlashStorage) -> u32 {
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

    return STORAGE_OFFSET + 4;
}
