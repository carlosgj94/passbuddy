use crate::keepass::header;
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
            signature1: u32::from_le_bytes(signature1_buffer),
            signature2: u32::from_le_bytes(signature2_buffer),
            header: header,
            groups: groups,
            entries: entries,
        })
    }
}
