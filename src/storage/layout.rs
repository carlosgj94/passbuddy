use defmt::Format;
use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::storage::{
    header::{
        LAYOUT_HEADER_SIZE, LayoutHeader, STORAGE_LAYOUT_VERSION, STORAGE_MAGIC,
        get_user_storage_offset, storage_magic_offset,
    },
    region::{DataRegion, REGION_DESCRIPTOR_SIZE, RegionDescriptor, RegionHandle},
};
use embedded_storage::Storage;
pub const REGION_COUNT: usize = 4;

const STORAGE_METADATA_BYTES: u32 = FlashStorage::SECTOR_SIZE;
const REGION_PROJECT_CAPACITY: u32 = FlashStorage::SECTOR_SIZE;
const REGION_USER_CONFIG_CAPACITY: u32 = FlashStorage::SECTOR_SIZE;
const REGION_KEEPASS_CAPACITY: u32 = 64 * 1024;
const REGION_SCRATCH_CAPACITY: u32 = FlashStorage::SECTOR_SIZE;

const STORAGE_TOTAL_BYTES: u32 = STORAGE_METADATA_BYTES
    + REGION_PROJECT_CAPACITY
    + REGION_USER_CONFIG_CAPACITY
    + REGION_KEEPASS_CAPACITY
    + REGION_SCRATCH_CAPACITY;

const fn region_capacity(region: DataRegion) -> u32 {
    match region {
        DataRegion::ProjectConfig => REGION_PROJECT_CAPACITY,
        DataRegion::UserConfig => REGION_USER_CONFIG_CAPACITY,
        DataRegion::KeePassDb => REGION_KEEPASS_CAPACITY,
        DataRegion::Scratch => REGION_SCRATCH_CAPACITY,
    }
}

fn expected_region_descriptors() -> [RegionDescriptor; REGION_COUNT] {
    let mut regions = [RegionDescriptor::empty(); REGION_COUNT];
    let mut next_offset = storage_magic_offset() + STORAGE_METADATA_BYTES;

    for (idx, region) in regions.iter_mut().enumerate() {
        let kind = match idx {
            0 => DataRegion::ProjectConfig,
            1 => DataRegion::UserConfig,
            2 => DataRegion::KeePassDb,
            3 => DataRegion::Scratch,
            _ => unreachable!("REGION_COUNT must match fixed region list"),
        };

        let capacity = region_capacity(kind);
        *region = RegionDescriptor {
            kind,
            offset: next_offset,
            capacity,
            used_len: 0,
            crc32: 0,
        };

        next_offset += capacity;
    }

    regions
}

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

        let layout_header = LayoutHeader::new_from_bytes(&header_buffer);
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
            header: layout_header,
            regions,
        }
    }
    pub fn run_healthcheck(storage: &mut FlashStorage) -> Result<(), StorageError> {
        // Ensure the declared storage window fits within flash.
        let capacity = storage.capacity() as u32;
        let start = storage_magic_offset();
        let end = start
            .checked_add(STORAGE_TOTAL_BYTES)
            .ok_or(StorageError::InvalidLayout)?;
        if end > capacity {
            return Err(StorageError::InvalidLayout);
        }

        let mut magic_buffer = [0u8; 4];
        storage
            .read(storage_magic_offset(), &mut magic_buffer)
            .expect("Read failed");

        if magic_buffer != STORAGE_MAGIC {
            return Err(StorageError::BadMagic);
        }

        let mut header_buffer = [0u8; LAYOUT_HEADER_SIZE];
        storage
            .read(get_user_storage_offset(), &mut header_buffer)
            .expect("header read failed");
        let header = LayoutHeader::new_from_bytes(&header_buffer);
        if header.magic != STORAGE_MAGIC {
            return Err(StorageError::InvalidLayout);
        }
        if header.layout_version != STORAGE_LAYOUT_VERSION {
            return Err(StorageError::UnsupportedLayout(header.layout_version));
        }
        if header.region_count != REGION_COUNT as u8 {
            return Err(StorageError::InvalidLayout);
        }

        // Validate region descriptors match the expected deterministic layout.
        let expected = expected_region_descriptors();
        let mut desc_offset = get_user_storage_offset() + LAYOUT_HEADER_SIZE as u32;
        for (idx, expected_desc) in expected.iter().enumerate() {
            let mut region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
            storage
                .read(desc_offset, &mut region_buffer)
                .expect("region descriptor read failed");
            let actual = RegionDescriptor::new_from_bytes(&region_buffer);
            if actual.kind != expected_desc.kind
                || actual.offset != expected_desc.offset
                || actual.capacity != expected_desc.capacity
            {
                return Err(StorageError::InvalidLayout);
            }

            // Basic overlap/alignment checks.
            if actual.offset % FlashStorage::SECTOR_SIZE != 0
                || actual.capacity % FlashStorage::SECTOR_SIZE != 0
            {
                return Err(StorageError::InvalidLayout);
            }
            let actual_end = actual
                .offset
                .checked_add(actual.capacity)
                .ok_or(StorageError::InvalidLayout)?;
            if actual.offset < start || actual_end > end {
                return Err(StorageError::InvalidLayout);
            }
            if idx > 0 {
                let prev = expected[idx - 1];
                let prev_end = prev
                    .offset
                    .checked_add(prev.capacity)
                    .ok_or(StorageError::InvalidLayout)?;
                if actual.offset < prev_end {
                    return Err(StorageError::InvalidLayout);
                }
            }

            desc_offset += REGION_DESCRIPTOR_SIZE as u32;
        }

        Ok(())
    }

    pub fn wipe_layout(storage: &mut FlashStorage) -> Result<(), StorageError> {
        use embedded_storage::nor_flash::NorFlash;

        let start = storage_magic_offset(); // 0x200000
        let end = start + FlashStorage::SECTOR_SIZE; // + 4KiB

        storage.erase(start, end).map_err(|_| StorageError::Io)
    }

    pub fn bootstrap_storage_write(storage: &mut FlashStorage) -> Result<(), StorageError> {
        let capacity = storage.capacity() as u32;
        let start = storage_magic_offset();
        let end = start
            .checked_add(STORAGE_TOTAL_BYTES)
            .ok_or(StorageError::InvalidLayout)?;
        if end > capacity {
            return Err(StorageError::InvalidLayout);
        }

        // Start from a clean slate; this project is early-stage so we prefer a hard reset.
        embedded_storage::nor_flash::NorFlash::erase(storage, start, end)
            .map_err(|_| StorageError::Io)?;

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

        // 4. Initialize regions deterministically (aligned and non-overlapping).
        let expected = expected_region_descriptors();
        let mut regions_offset = get_user_storage_offset() + LAYOUT_HEADER_SIZE as u32;
        for desc in expected {
            storage
                .write(regions_offset, &desc.to_bytes())
                .expect("region descriptor write failed");
            regions_offset += REGION_DESCRIPTOR_SIZE as u32;
        }

        Ok(())
    }

    pub fn get_offset_to_region(&self, region: DataRegion) -> Result<u32, StorageError> {
        let idx = region.index();
        self.regions
            .get(idx)
            .map(|desc| desc.offset)
            .ok_or(StorageError::RegionNotFound)
    }

    pub fn region_handle(&self, region: DataRegion) -> Result<RegionHandle, StorageError> {
        let idx = region.index();
        let desc = *self.regions.get(idx).ok_or(StorageError::RegionNotFound)?;
        Ok(RegionHandle {
            base: desc.offset,
            capacity: desc.capacity,
        })
    }

    pub fn get_offset_to_keepass(&self) -> u32 {
        self.get_offset_to_region(DataRegion::KeePassDb)
            .unwrap_or_else(|_| {
                storage_magic_offset()
                    + STORAGE_METADATA_BYTES
                    + REGION_PROJECT_CAPACITY
                    + REGION_USER_CONFIG_CAPACITY
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Format)]
pub enum StorageError {
    BadMagic,
    UnsupportedLayout(u16),
    RegionNotFound,
    BufferTooSmall,
    Io,
    InvalidLayout,
}
