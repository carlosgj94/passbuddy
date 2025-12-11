use defmt::Format;
use esp_storage::FlashStorage;

pub const STORAGE_MAGIC: [u8; 4] = *b"PBDY";
pub const STORAGE_LAYOUT_VERSION: u16 = 1;

/// Regions we plan to keep in flash. Add more as the layout evolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
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
    pub len: u32,
    pub crc32: u32,
}

/// Small header to sit ahead of the descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct LayoutHeader {
    pub magic: [u8; 4],
    pub layout_version: u16,
    pub region_count: u8,
}

/// Fixed set of descriptors baked into firmware for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct StorageLayout<const N: usize> {
    pub header: LayoutHeader,
    pub regions: [RegionDescriptor; N],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum StorageError {
    BadMagic,
    UnsupportedLayout(u16),
    RegionNotFound,
    BufferTooSmall,
    Io,
}

/// Return the expected header. For now this is a constant; later it can read from flash.
pub fn get_header(_flash: &mut FlashStorage) -> LayoutHeader {
    LayoutHeader {
        magic: STORAGE_MAGIC,
        layout_version: STORAGE_LAYOUT_VERSION,
        region_count: 4, // ProjectConfig, UserConfig, KeePassDb, Scratch
    }
}
