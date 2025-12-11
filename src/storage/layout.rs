use core::mem;

use defmt::Format;
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::storage::{
    header::{
        LayoutHeader, STORAGE_LAYOUT_VERSION, get_layout_header_from_bytes, get_user_storage_offset,
    },
    region::{RegionDescriptor, get_region_descriptor_from_bytes},
};

pub const REGION_COUNT: usize = 4;

/// Fixed set of descriptors baked into firmware for now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub struct StorageLayout {
    pub header: LayoutHeader,
    pub regions: [RegionDescriptor; REGION_COUNT],
}

impl StorageLayout {
    pub fn new(storage: &mut FlashStorage) -> Self {
        // 1. Go to the storage layout and ensure we read the header
        let mut offset = get_user_storage_offset(storage);

        // 2. Read the header from the storage layout
        let mut header_buffer = [0u8; mem::size_of::<LayoutHeader>()];
        storage
            .read(offset, &mut header_buffer)
            .expect("Read failed");

        let layout_header = get_layout_header_from_bytes(&header_buffer);
        offset += mem::size_of::<LayoutHeader>() as u32;

        // 3. Read the fixed set of region descriptors
        let mut current_offset = offset;
        let mut regions = [RegionDescriptor::empty(); REGION_COUNT];
        for idx in 0..REGION_COUNT {
            let mut region_buffer = [0u8; mem::size_of::<RegionDescriptor>()];
            storage
                .read(current_offset, &mut region_buffer)
                .expect("region reading failed");

            current_offset += mem::size_of::<RegionDescriptor>() as u32;
            regions[idx] = get_region_descriptor_from_bytes(&region_buffer);
        }

        Self {
            header: LayoutHeader {
                magic: super::header::STORAGE_MAGIC,
                layout_version: STORAGE_LAYOUT_VERSION,
                region_count: REGION_COUNT as u8,
                ..layout_header
            },
            regions,
        }
    }
    pub fn run_healthcheck(storage: &mut FlashStorage) -> Result<(), StorageError> {
        let mut offset = get_user_storage_offset(storage);

        // 2. Read the header from the storage layout
        let mut header_buffer = [0u8; mem::size_of::<LayoutHeader>()];
        storage
            .read(offset, &mut header_buffer)
            .expect("Read failed");

        // 3. We check the header magic is correct
        todo!();
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
