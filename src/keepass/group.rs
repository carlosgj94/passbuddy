use super::times::Times;
use defmt::Format;

// group_id = 4; name = 64; icon_id = 8; level = 2; times = 20; padding = 2;
pub const GROUP_SIZE: usize = 4 + 64 + 8 + 2 + 20 + 2; // 100

#[derive(Clone, Copy, Format, Debug)]
pub struct Group {
    /// The unique identifier of the group
    pub group_id: u32,

    /// The name of the group
    pub name: [u8; 64],

    /// ID of the group's icon
    pub icon_id: Option<u32>,

    /// Level of the group in the hierarchy
    pub level: u16,

    /// The list of time fields for this group
    pub times: Times,
}

impl Group {
    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let group_id = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let name: [u8; 64] = bytes[4..68].try_into().unwrap();
        let icon_id: Option<u32> = Some(u32::from_le_bytes(bytes[68..72].try_into().unwrap()));
        let level = u16::from_le_bytes(bytes[76..78].try_into().unwrap());
        let times = Times::new_from_bytes(&bytes[78..98]);

        Self {
            group_id,
            name,
            icon_id,
            level,
            times,
        }
    }

    pub fn random() -> Self {
        let mut name = [0u8; 64];
        name[..b"Private".len()].copy_from_slice(b"Private");

        Self {
            group_id: 1,
            name,
            icon_id: None,
            level: 0,
            times: Times::zero(),
        }
    }

    pub fn to_bytes(&self) -> [u8; GROUP_SIZE] {
        let mut bytes = [0u8; GROUP_SIZE];
        bytes[0..4].copy_from_slice(&self.group_id.to_le_bytes());
        bytes[4..68].copy_from_slice(&self.name);
        bytes[68..72].copy_from_slice(&self.icon_id.unwrap_or(0).to_le_bytes());
        bytes[76..78].copy_from_slice(&self.level.to_le_bytes());
        bytes[78..98].copy_from_slice(&self.times.to_bytes());
        bytes
    }
}
