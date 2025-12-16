use defmt::Format;
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::storage::{
    header::{
        LAYOUT_HEADER_SIZE, LayoutHeader, STORAGE_LAYOUT_VERSION, STORAGE_MAGIC,
        get_user_storage_offset, storage_magic_offset,
    },
    region::{DataRegion, REGION_DESCRIPTOR_SIZE, RegionDescriptor},
};
use embedded_storage::Storage;

const STORAGE_OFFSET: u32 = 0x200000;
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
        let mut offset = get_user_storage_offset();

        // 2. Read the header from the storage layout
        let mut header_buffer = [0u8; LAYOUT_HEADER_SIZE];
        storage
            .read(offset, &mut header_buffer)
            .expect("Read failed");

        let _layout_header = LayoutHeader::new_from_bytes(&header_buffer);
        offset += LAYOUT_HEADER_SIZE as u32;

        // 3. Read the fixed set of region descriptors
        let mut current_offset = offset;
        let mut regions = [RegionDescriptor::empty(); REGION_COUNT];
        for region in &mut regions {
            let mut region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
            storage
                .read(current_offset, &mut region_buffer)
                .expect("region reading failed");

            current_offset += REGION_DESCRIPTOR_SIZE as u32;
            *region = RegionDescriptor::new_from_bytes(&region_buffer);
        }

        Self {
            header: LayoutHeader {
                magic: super::header::STORAGE_MAGIC,
                layout_version: STORAGE_LAYOUT_VERSION,
                region_count: REGION_COUNT as u8,
            },
            regions,
        }
    }
    pub fn run_healthcheck(storage: &mut FlashStorage) -> Result<(), StorageError> {
        let mut magic_buffer = [0u8; 4];
        storage
            .read(storage_magic_offset(), &mut magic_buffer)
            .expect("Read failed");

        if magic_buffer != STORAGE_MAGIC {
            return Err(StorageError::BadMagic);
        }

        // For now we only check the magic number, if it's correct, then we good
        Ok(())
    }

    pub fn bootstrap_storage_write(storage: &mut FlashStorage) -> Result<(), StorageError> {
        storage
            .write(storage_magic_offset(), &STORAGE_MAGIC)
            .expect("Storage magic write failed");

        // 2. Create the header
        let header = LayoutHeader {
            magic: super::header::STORAGE_MAGIC,
            layout_version: STORAGE_LAYOUT_VERSION,
            region_count: REGION_COUNT as u8,
        };

        // 3. Write the header to the storage layout
        storage
            .write(get_user_storage_offset(), &header.get_bytes())
            .expect("Write failed");

        // 4. Initialize the regions
        let mut regions_offset = get_user_storage_offset();

        // TODO: Assign real region `offset`/`capacity` values (aligned and non-overlapping).
        let project_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::ProjectConfig);
        let user_config_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::UserConfig);
        let keepass_region =
            RegionDescriptor::empty_with_kind(super::region::DataRegion::KeePassDb);
        let scratch_region = RegionDescriptor::empty_with_kind(super::region::DataRegion::Scratch);

        // 5. Write the regions to the storage layout
        regions_offset += LAYOUT_HEADER_SIZE as u32;
        storage
            .write(regions_offset, &project_region.to_bytes())
            .expect("project region write failed");
        regions_offset += REGION_DESCRIPTOR_SIZE as u32;

        storage
            .write(regions_offset, &user_config_region.to_bytes())
            .expect("user config region write failed");
        regions_offset += REGION_DESCRIPTOR_SIZE as u32;

        storage
            .write(regions_offset, &keepass_region.to_bytes())
            .expect("keepass region write failed");
        regions_offset += REGION_DESCRIPTOR_SIZE as u32;

        storage
            .write(regions_offset, &scratch_region.to_bytes())
            .expect("scratch region write failed");

        Ok(())
    }

    pub fn get_offset_to_region(&self, region: DataRegion) -> Result<u32, StorageError> {
        let offset_to_regions =
            STORAGE_OFFSET + (LAYOUT_HEADER_SIZE + (REGION_DESCRIPTOR_SIZE * 4)) as u32;
        match region {
            DataRegion::ProjectConfig => Ok(offset_to_regions),
            DataRegion::UserConfig => Ok(offset_to_regions + self.regions[0].capacity),
            DataRegion::KeePassDb => {
                Ok(offset_to_regions + self.regions[0].capacity + self.regions[1].capacity)
            }
            DataRegion::Scratch => Ok(offset_to_regions
                + self.regions[0].capacity
                + self.regions[1].capacity
                + self.regions[2].capacity),
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
