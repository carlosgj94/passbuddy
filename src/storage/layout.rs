use core::mem;

use defmt::Format;
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::storage::{
    header::{LayoutHeader, STORAGE_LAYOUT_VERSION, STORAGE_MAGIC, get_user_storage_offset},
    region::RegionDescriptor,
};
use embedded_storage::Storage;

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

        let layout_header = LayoutHeader::new_from_bytes(&header_buffer);
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
            regions[idx] = RegionDescriptor::new_from_bytes(&region_buffer);
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
        let offset = get_user_storage_offset(storage);

        // 2. Read the header from the storage layout
        let mut magic_buffer = [0u8; 4];
        storage
            .read(offset, &mut magic_buffer)
            .expect("Read failed");

        // 3. We check the header magic is correct
        if magic_buffer != &STORAGE_MAGIC[..] {
            return Err(StorageError::BadMagic);
        }

        // For now we only check the magic number, if it's correct, then we good
        Ok(())
    }

    pub fn bootstrap_storage_write(storage: &mut FlashStorage) -> Result<(), StorageError> {
        let offset = get_user_storage_offset(storage);

        // 1. First thing is writing the magic number
        storage
            .write(offset, &STORAGE_MAGIC)
            .expect("Storage magic write failed");

        let header_offset = offset + mem::size_of::<u32>() as u32;

        // 2. Create the header
        let header = LayoutHeader {
            magic: super::header::STORAGE_MAGIC,
            layout_version: STORAGE_LAYOUT_VERSION,
            region_count: REGION_COUNT as u8,
        };

        // 3. Write the header to the storage layout
        storage
            .write(header_offset, &header.get_bytes())
            .expect("Write failed");

        // 4. Initialize the regions
        let mut regions_offset = header_offset;

        let project_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::ProjectConfig);
        let user_config_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::UserConfig);
        let keepass_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::KeePassDb);
        let scratch_region = RegionDescriptor::empty_with_kind(super::region::DataRegion::Scratch);

        // 5. Write the regions to the storage layout
        regions_offset += mem::size_of::<LayoutHeader>() as u32;
        storage
            .write(regions_offset, &project_region.to_bytes())
            .expect("project region write failed");
        regions_offset += project_region.to_bytes().len() as u32;

        storage
            .write(regions_offset, &user_config_region.to_bytes())
            .expect("user config region write failed");
        regions_offset += user_config_region.to_bytes().len() as u32;

        storage
            .write(regions_offset, &keepass_region.to_bytes())
            .expect("keepass region write failed");
        regions_offset += keepass_region.to_bytes().len() as u32;

        storage
            .write(regions_offset, &scratch_region.to_bytes())
            .expect("scratch region write failed");

        Ok(())
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
