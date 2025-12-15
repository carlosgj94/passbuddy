use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::keepass::{
    Entry, Group, HEADER_SIZE, KDBError, KDBHeader, KeePassDb,
    entry::ENTRY_SIZE,
    group::GROUP_SIZE,
    header::{KDB_SIGNATURE1, KDB_SIGNATURE2},
};

impl<const N: usize, const M: usize> KeePassDb<N, M> {
    pub fn new(storage: &mut FlashStorage, offset: u32) -> Result<Self, KDBError> {
        // 1. we check the magic signatures are there
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

        // 2. We get the header
        let header_offset = offset + 8;
        let mut header_buffer = [0u8; HEADER_SIZE];
        storage.read(header_offset, &mut header_buffer).unwrap();
        let header = KDBHeader::new_from_header(&header_buffer)?;

        // 3. We get the groups
        let mut group_offset = header_offset + HEADER_SIZE as u32;
        let mut group_buffer = [0u8; GROUP_SIZE];
        let mut groups: [Option<Group>; N] = [None; N];
        for i in 0..header.num_groups as usize {
            storage.read(group_offset, &mut group_buffer).unwrap();
            groups[i] = Some(Group::new_from_bytes(&group_buffer));
            group_offset += GROUP_SIZE as u32;
        }
        // 4. We get the entries
        let mut entry_offset = group_offset;
        let mut entry_buffer = [0u8; ENTRY_SIZE];
        let mut entries: [Option<Entry>; M] = [None; M];
        for i in 0..header.num_entries as usize {
            storage.read(entry_offset, &mut entry_buffer).unwrap();
            entries[i] = Some(Entry::new_from_bytes(&entry_buffer));
            entry_offset += ENTRY_SIZE as u32;
        }
        // 5. Return the database
        Ok(KeePassDb::<N, M> {
            signature1: u32::from_le_bytes(signature1_buffer),
            signature2: u32::from_le_bytes(signature2_buffer),
            header: header,
            groups: groups,
            entries: entries,
        })
    }
}
