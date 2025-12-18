use super::times::Times;
use defmt::Format;

// group_id = 4; name = 64; times = 20;
pub const GROUP_SIZE: usize = 4 + 64 + 20; // 88

#[derive(Clone, Copy, Format, Debug)]
pub struct Group {
    /// The unique identifier of the group
    pub group_id: u32,

    /// The name of the group
    pub name: [u8; 64],

    /// The list of time fields for this group
    pub times: Times,
}

impl Group {
    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let group_id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let name: [u8; 64] = bytes[4..68].try_into().unwrap();
        let times = Times::new_from_bytes(&bytes[68..88]);

        Self {
            group_id,
            name,
            times,
        }
    }

    pub fn random() -> Self {
        let mut name = [0u8; 64];
        name[..b"Private".len()].copy_from_slice(b"Private");

        Self {
            group_id: 1,
            name,
            times: Times::zero(),
        }
    }

    pub fn to_bytes(&self) -> [u8; GROUP_SIZE] {
        let mut bytes = [0u8; GROUP_SIZE];
        bytes[0..4].copy_from_slice(&self.group_id.to_le_bytes());
        bytes[4..68].copy_from_slice(&self.name);
        bytes[68..88].copy_from_slice(&self.times.to_bytes());
        bytes
    }
}
