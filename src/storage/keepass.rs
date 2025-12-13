use embedded_storage::ReadStorage;
use esp_storage::FlashStorage;

use crate::keepass::{
    KDBError, KeePassDb,
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
        let _header_offset = offset + 8;

        // 3. We get the groups
        // 4. We get the entries
        // 5. Return the database

        todo!();
    }
}
