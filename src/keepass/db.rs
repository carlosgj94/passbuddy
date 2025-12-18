use super::{Entry, KDBHeader};
use crate::keepass::group::Group; // or your slim v1 Group type
use crate::storage::region::RegionHandle;

// TODO: Make the lenght configurable through a const
#[derive(Debug, Clone)]
pub struct KeePassDb {
    pub storage: RegionHandle,
    pub signature1: u32, // expect 0x9AA2D903
    pub signature2: u32, // expect 0xB54BFB65
    pub header: KDBHeader,
    pub groups: [Option<Group>; 4],
    pub entries: [Option<Entry>; 256],
}
