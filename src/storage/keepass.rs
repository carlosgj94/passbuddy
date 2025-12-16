use crate::storage::header::{LAYOUT_HEADER_SIZE, get_user_storage_offset};
use crate::storage::region::{DataRegion, REGION_DESCRIPTOR_SIZE, RegionDescriptor};
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

impl KeePassDb {
    pub fn check_if_exists(storage: &mut FlashStorage, offset: u32) -> Result<bool, KDBError> {
        // 1. we check the magic signatures are there
        let mut signature1_buffer = [0u8; 4];
        storage.read(offset, &mut signature1_buffer).unwrap();
        if signature1_buffer != KDB_SIGNATURE1.to_le_bytes() {
            return Ok(false);
        }
        let mut signature2_buffer = [0u8; 4];
        storage.read(offset + 4, &mut signature2_buffer).unwrap();
        if signature2_buffer != KDB_SIGNATURE2.to_le_bytes() {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn initialize_db(storage: &mut FlashStorage, offset: u32) -> Result<(), KDBError> {
        info!("---- offset: {:?}", offset);
        // 1. Initialize the database by writing the magic signatures
        let mut signature1_buffer = [0u8; 4];
        signature1_buffer.copy_from_slice(&KDB_SIGNATURE1.to_le_bytes());
        storage.write(offset, &signature1_buffer).unwrap();
        info!("---- Signature1: {:?}", &signature1_buffer);
        let mut signature2_buffer = [0u8; 4];
        signature2_buffer.copy_from_slice(&KDB_SIGNATURE2.to_le_bytes());
        storage.write(offset + 4, &signature2_buffer).unwrap();
        info!("---- Signature2: {:?}", &signature2_buffer);

        // 2. We add the header
        let header = KDBHeader::empty();
        info!("---- Header: {:?}", &header.to_bytes());
        storage.write(offset + 8, &header.to_bytes()).unwrap();
        Ok(())
    }

    pub fn new(storage: &mut FlashStorage, offset: u32) -> Result<Self, KDBError> {
        // 1. we check the magic signatures are there
        info!("Getting the magic signatures");
        let mut signature1_buffer = [0u8; 4];
        storage.read(offset, &mut signature1_buffer).unwrap();
        if signature1_buffer != KDB_SIGNATURE1.to_le_bytes() {
            return Err(KDBError::DatabaseIntegrityError);
        }
        let mut signature2_buffer = [0u8; 4];
        storage.read(offset + 4, &mut signature2_buffer).unwrap();
        if signature2_buffer != KDB_SIGNATURE2.to_le_bytes() {
            return Err(KDBError::DatabaseIntegrityError);
        }
        info!("Signature1 buffer: {:?}", signature1_buffer);
        info!("Signature2 buffer: {:?}", signature2_buffer);

        // 2. We get the header
        info!("Getting the header");
        let header_offset = offset + 8;
        let mut header_buffer = [0u8; HEADER_SIZE];
        storage.read(header_offset, &mut header_buffer).unwrap();
        let header = KDBHeader::new_from_bytes(&header_buffer)?;
        info!("Header: {}", header);

        // 3. We get the groups
        info!("Getting the groups");
        let mut group_offset = header_offset + HEADER_SIZE as u32;
        let mut group_buffer = [0u8; GROUP_SIZE];
        let mut groups: [Option<Group>; 4] = [None; 4];
        for i in 0..header.num_groups as usize {
            storage.read(group_offset, &mut group_buffer).unwrap();
            groups[i] = Some(Group::new_from_bytes(&group_buffer));
            group_offset += GROUP_SIZE as u32;
        }
        info!("Groups: {:?}", groups);

        // 4. We get the entries
        info!("Getting the entries");
        let mut entry_offset = group_offset;
        let mut entry_buffer = [0u8; ENTRY_SIZE];
        let mut entries: [Option<Entry>; 128] = [None; 128];
        for i in 0..header.num_entries as usize {
            storage.read(entry_offset, &mut entry_buffer).unwrap();
            entries[i] = Some(Entry::new_from_bytes(&entry_buffer));
            entry_offset += ENTRY_SIZE as u32;
        }
        info!("Entries: {:?}", entries);

        // 5. Return the database
        info!("Returning database");
        Ok(KeePassDb {
            storage_offset: offset,
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
        if self.header.num_groups >= 4 {
            return Err(KDBError::DatabaseIntegrityError);
        }

        let group_index = self.header.num_groups;
        let group_offset =
            self.storage_offset + 8 + HEADER_SIZE as u32 + (group_index * GROUP_SIZE as u32);
        let group_bytes = group.to_bytes();

        // 1. Write group contents.
        storage.write(group_offset, &group_bytes).unwrap();

        // 2. Update internal region descriptor (best-effort bookkeeping).
        let keepass_region_offset = get_user_storage_offset()
            + LAYOUT_HEADER_SIZE as u32
            + (REGION_DESCRIPTOR_SIZE as u32 * DataRegion::KeePassDb as u32);
        let mut keepass_region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
        storage
            .read(keepass_region_offset, &mut keepass_region_buffer)
            .unwrap();
        let mut keepass_region = RegionDescriptor::new_from_bytes(&keepass_region_buffer);
        keepass_region.used_len = keepass_region.used_len.saturating_add(GROUP_SIZE as u32);
        storage
            .write(keepass_region_offset, &keepass_region.to_bytes())
            .unwrap();

        // 3. Update and persist the KeePass header.
        self.header.num_groups += 1;
        let keepass_header_offset = self.storage_offset + 8;
        storage
            .write(keepass_header_offset, &self.header.to_bytes())
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
        if self.header.num_entries >= 128 {
            return Err(KDBError::DatabaseIntegrityError);
        }

        // 1. Calculate the offset for the new entry
        let entry_index = self.header.num_entries;
        let entry_offset = self.storage_offset // Keepass offset
            + 8 // two magic signatures
            + HEADER_SIZE as u32 // This is the keepass header
            + (4 * GROUP_SIZE) as u32 // Taking as much space for groups to avoid collisions
            + (entry_index * ENTRY_SIZE as u32);
        let mut entry_bytes = entry.to_bytes();

        // 2. Write the entry contents
        storage.write(entry_offset, &mut entry_bytes).unwrap();

        // 3. Update internal region descriptors (for bookkeeping)
        let keepass_region_offset = get_user_storage_offset()
            + LAYOUT_HEADER_SIZE as u32
            + (REGION_DESCRIPTOR_SIZE as u32 * DataRegion::KeePassDb as u32);
        let mut keepass_region_buffer = [0u8; REGION_DESCRIPTOR_SIZE];
        storage
            .read(keepass_region_offset, &mut keepass_region_buffer)
            .unwrap();
        let mut keepass_region = RegionDescriptor::new_from_bytes(&keepass_region_buffer);
        keepass_region.used_len = keepass_region.used_len.saturating_add(ENTRY_SIZE as u32);
        storage
            .write(keepass_region_offset, &keepass_region.to_bytes())
            .unwrap();

        // 4. Update and persist the keepass header
        self.header.num_entries += 1;
        let keepass_header_offset = self.storage_offset + 8;
        storage
            .write(keepass_header_offset, &self.header.to_bytes())
            .unwrap();

        // 4. Update in-memory cache.
        self.entries[entry_index as usize] = Some(entry);

        info!("Created entry at offset {}", entry_offset);
        Ok(())
    }
}
