use defmt::Format;

use crate::storage::header::{LayoutHeader, STORAGE_LAYOUT_VERSION};

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

/// Fixed set of descriptors baked into firmware for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct StorageLayout<const N: usize> {
    pub header: LayoutHeader,
    pub regions: [RegionDescriptor; N],
}

impl<const N: usize> StorageLayout<N> {
    pub const fn new(regions: [RegionDescriptor; N]) -> Self {
        Self {
            header: LayoutHeader {
                magic: super::header::STORAGE_MAGIC,
                layout_version: STORAGE_LAYOUT_VERSION,
                region_count: N as u8,
            },
            regions,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum StorageError {
    BadMagic,
    UnsupportedLayout(u16),
    RegionNotFound,
    BufferTooSmall,
    Io,
}
