use super::{Entry, KDBHeader};
use crate::keepass::group::Group; // or your slim v1 Group type

pub struct KeePassDb<const MAX_GROUPS: usize, const MAX_ENTRIES: usize> {
    pub signature1: u32, // expect 0x9AA2D903
    pub signature2: u32, // expect 0xB54BFB65
    pub header: KDBHeader,
    pub groups: [Option<Group>; MAX_GROUPS],
    pub entries: [Option<Entry>; MAX_ENTRIES],
}
