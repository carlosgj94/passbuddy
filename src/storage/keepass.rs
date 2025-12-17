use crate::storage::header::{LAYOUT_HEADER_SIZE, get_user_storage_offset};
use crate::storage::region::{DataRegion, REGION_DESCRIPTOR_SIZE, RegionDescriptor, RegionHandle};
use defmt::info;
use embedded_storage::ReadStorage;
use embedded_storage::Storage;
use esp_storage::FlashStorage;

use crate::keepass::{
    Entry, Group, HEADER_SIZE, KDBError, KDBHeader, KeePassDb,
    entry::ENTRY_SIZE,
    group::GROUP_SIZE,
    header::{KDB_SIGNATURE1, KDB_SIGNATURE2},
};

const SIGNATURE1_OFFSET_REL: u32 = 0;
const SIGNATURE2_OFFSET_REL: u32 = 4;
const HEADER_OFFSET_REL: u32 = 8;
const MAX_GROUPS: u32 = 4;
const MAX_ENTRIES: u32 = 128;

const fn groups_offset_rel() -> u32 {
    HEADER_OFFSET_REL + HEADER_SIZE as u32
}

const fn entries_offset_rel() -> u32 {
    groups_offset_rel() + (MAX_GROUPS * GROUP_SIZE as u32)
}

fn checked_absolute(
    region: RegionHandle,
    relative_offset: u32,
    len: usize,
) -> Result<u32, KDBError> {
    if !region.contains_range(relative_offset, len) {
        return Err(KDBError::DatabaseIntegrityError);
    }
    region
        .absolute(relative_offset)
        .ok_or(KDBError::DatabaseIntegrityError)
}

fn descriptor_offset(kind: DataRegion) -> u32 {
    get_user_storage_offset()
        + LAYOUT_HEADER_SIZE as u32
        + (REGION_DESCRIPTOR_SIZE as u32 * kind.index() as u32)
}

impl KeePassDb {
    pub fn check_if_exists(
        storage: &mut FlashStorage,
        region: RegionHandle,
    ) -> Result<bool, KDBError> {
        // 1. we check the magic signatures are there
        let mut signature1_buffer = [0u8; 4];
        storage
            .read(
                checked_absolute(region, SIGNATURE1_OFFSET_REL, signature1_buffer.len())?,
                &mut signature1_buffer,
            )
            .unwrap();
        if signature1_buffer != KDB_SIGNATURE1.to_le_bytes() {
            return Ok(false);
        }
        let mut signature2_buffer = [0u8; 4];
        storage
            .read(
                checked_absolute(region, SIGNATURE2_OFFSET_REL, signature2_buffer.len())?,
                &mut signature2_buffer,
            )
            .unwrap();
        if signature2_buffer != KDB_SIGNATURE2.to_le_bytes() {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn initialize_db(storage: &mut FlashStorage, region: RegionHandle) -> Result<(), KDBError> {
        info!("---- offset: {:?}", region.base);
        // 1. Initialize the database by writing the magic signatures
        let mut signature1_buffer = [0u8; 4];
        signature1_buffer.copy_from_slice(&KDB_SIGNATURE1.to_le_bytes());
        storage
            .write(
                checked_absolute(region, SIGNATURE1_OFFSET_REL, signature1_buffer.len())?,
                &signature1_buffer,
            )
            .unwrap();
        info!("---- Signature1: {:?}", &signature1_buffer);
        let mut signature2_buffer = [0u8; 4];
        signature2_buffer.copy_from_slice(&KDB_SIGNATURE2.to_le_bytes());
        storage
            .write(
                checked_absolute(region, SIGNATURE2_OFFSET_REL, signature2_buffer.len())?,
                &signature2_buffer,
            )
            .unwrap();
        info!("---- Signature2: {:?}", &signature2_buffer);

        // 2. We add the header
        let header = KDBHeader::empty();
        info!("---- Header: {:?}", &header.to_bytes());
        storage
            .write(
                checked_absolute(region, HEADER_OFFSET_REL, HEADER_SIZE)?,
                &header.to_bytes(),
            )
            .unwrap();
        Ok(())
    }

    pub fn new(storage: &mut FlashStorage, region: RegionHandle) -> Result<Self, KDBError> {
        // 1. we check the magic signatures are there
        info!("Getting the magic signatures");
        let mut signature1_buffer = [0u8; 4];
        storage
            .read(
                checked_absolute(region, SIGNATURE1_OFFSET_REL, signature1_buffer.len())?,
                &mut signature1_buffer,
            )
            .unwrap();
        if signature1_buffer != KDB_SIGNATURE1.to_le_bytes() {
            return Err(KDBError::DatabaseIntegrityError);
        }
        let mut signature2_buffer = [0u8; 4];
        storage
            .read(
                checked_absolute(region, SIGNATURE2_OFFSET_REL, signature2_buffer.len())?,
                &mut signature2_buffer,
            )
            .unwrap();
        if signature2_buffer != KDB_SIGNATURE2.to_le_bytes() {
            return Err(KDBError::DatabaseIntegrityError);
        }
        info!("Signature1 buffer: {:?}", signature1_buffer);
        info!("Signature2 buffer: {:?}", signature2_buffer);

        // 2. We get the header
        info!("Getting the header");
        let mut header_buffer = [0u8; HEADER_SIZE];
        storage
            .read(
                checked_absolute(region, HEADER_OFFSET_REL, header_buffer.len())?,
                &mut header_buffer,
            )
            .unwrap();
        let header = KDBHeader::new_from_bytes(&header_buffer)?;
        info!("Header: {}", header);
        if header.num_groups > MAX_GROUPS || header.num_entries > MAX_ENTRIES {
            return Err(KDBError::DatabaseIntegrityError);
        }

        // 3. We get the groups
        info!("Getting the groups");
        let mut group_buffer = [0u8; GROUP_SIZE];
        let mut groups: [Option<Group>; 4] = [None; 4];
        for i in 0..header.num_groups as usize {
            storage
                .read(
                    checked_absolute(
                        region,
                        groups_offset_rel() + (i as u32 * GROUP_SIZE as u32),
                        group_buffer.len(),
                    )?,
                    &mut group_buffer,
                )
                .unwrap();
            groups[i] = Some(Group::new_from_bytes(&group_buffer));
        }
        info!("Groups: {:?}", groups);

        // 4. We get the entries
        info!("Getting the entries");
        let mut entry_buffer = [0u8; ENTRY_SIZE];
        let mut entries: [Option<Entry>; 128] = [None; 128];
        for i in 0..header.num_entries as usize {
            storage
                .read(
                    checked_absolute(
                        region,
                        entries_offset_rel() + (i as u32 * ENTRY_SIZE as u32),
                        entry_buffer.len(),
                    )?,
                    &mut entry_buffer,
                )
                .unwrap();
            entries[i] = Some(Entry::new_from_bytes(&entry_buffer));
        }
        info!("Entries: {:?}", entries);

        // 5. Return the database
        info!("Returning database");
        Ok(KeePassDb {
            storage: region,
            signature1: u32::from_le_bytes(signature1_buffer),
            signature2: u32::from_le_bytes(signature2_buffer),
            header: header,
            groups: groups,
            entries: entries,
        })
    }

    pub fn create_group(
        &mut self,
        group: Group,
        storage: &mut FlashStorage,
    ) -> Result<(), KDBError> {
        if self.header.num_groups >= MAX_GROUPS {
            return Err(KDBError::DatabaseIntegrityError);
        }

        let group_index = self.header.num_groups;
        let group_offset = checked_absolute(
            self.storage,
            groups_offset_rel() + (group_index * GROUP_SIZE as u32),
            GROUP_SIZE,
        )?;
        let group_bytes = group.to_bytes();

        // 1. Write group contents.
        storage.write(group_offset, &group_bytes).unwrap();

        // 2. Update internal region descriptor (best-effort bookkeeping).
        let keepass_region_offset = descriptor_offset(DataRegion::KeePassDb);
        let mut keepass_region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
        storage
            .read(keepass_region_offset, &mut keepass_region_buffer)
            .unwrap();
        let mut keepass_region = RegionDescriptor::new_from_bytes(&keepass_region_buffer);
        let written_end = groups_offset_rel() + ((group_index + 1) * GROUP_SIZE as u32);
        keepass_region.used_len = keepass_region.used_len.max(written_end);
        storage
            .write(keepass_region_offset, &keepass_region.to_bytes())
            .unwrap();

        // 3. Update and persist the KeePass header.
        self.header.num_groups += 1;
        storage
            .write(
                checked_absolute(self.storage, HEADER_OFFSET_REL, HEADER_SIZE)?,
                &self.header.to_bytes(),
            )
            .unwrap();

        // 4. Update in-memory cache.
        self.groups[group_index as usize] = Some(group);

        info!("Created group at offset {}", group_offset);
        Ok(())
    }

    pub fn create_entry(
        &mut self,
        entry: Entry,
        storage: &mut FlashStorage,
    ) -> Result<(), KDBError> {
        if self.header.num_entries >= MAX_ENTRIES {
            return Err(KDBError::DatabaseIntegrityError);
        }

        // 1. Calculate the offset for the new entry.
        let entry_index = self.header.num_entries;
        let entry_offset = checked_absolute(
            self.storage,
            entries_offset_rel() + (entry_index * ENTRY_SIZE as u32),
            ENTRY_SIZE,
        )?;
        let mut entry_bytes = entry.to_bytes();

        // 2. Write the entry contents
        storage.write(entry_offset, &mut entry_bytes).unwrap();

        // 3. Update internal region descriptors (for bookkeeping)
        let keepass_region_offset = descriptor_offset(DataRegion::KeePassDb);
        let mut keepass_region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
        storage
            .read(keepass_region_offset, &mut keepass_region_buffer)
            .unwrap();
        let mut keepass_region = RegionDescriptor::new_from_bytes(&keepass_region_buffer);
        let written_end = entries_offset_rel() + ((entry_index + 1) * ENTRY_SIZE as u32);
        keepass_region.used_len = keepass_region.used_len.max(written_end);
        storage
            .write(keepass_region_offset, &keepass_region.to_bytes())
            .unwrap();

        // 4. Update and persist the keepass header
        self.header.num_entries += 1;
        storage
            .write(
                checked_absolute(self.storage, HEADER_OFFSET_REL, HEADER_SIZE)?,
                &self.header.to_bytes(),
            )
            .unwrap();

        // 4. Update in-memory cache.
        self.entries[entry_index as usize] = Some(entry);

        info!("Created entry at offset {}", entry_offset);
        Ok(())
    }

    pub fn update_entry(
        &mut self,
        entry_index: usize,
        entry: Entry,
        storage: &mut FlashStorage,
    ) -> Result<(), KDBError> {
        if entry_index >= MAX_ENTRIES as usize {
            return Err(KDBError::DatabaseIntegrityError);
        }
        if entry_index >= self.header.num_entries as usize {
            return Err(KDBError::DatabaseIntegrityError);
        }

        let relative_offset = entries_offset_rel() + ((entry_index as u32) * ENTRY_SIZE as u32);
        let entry_offset = checked_absolute(self.storage, relative_offset, ENTRY_SIZE)?;

        let entry_bytes = entry.to_bytes();
        storage
            .write(entry_offset, &entry_bytes)
            .map_err(|_| KDBError::DatabaseIntegrityError)?;

        self.entries[entry_index] = Some(entry);
        Ok(())
    }
}
