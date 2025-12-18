use super::times::Times;

use defmt::Format;

// uuid = 16; group_id = 4; title = 64; username = 64; password = 64;
// times = 20; autotype = 1; padding = 3;
pub const ENTRY_SIZE: usize = 16 + 4 + 64 + 64 + 64 + 20 + 1 + 3; // 236

#[derive(Clone, Copy, Format, Debug)]
pub struct Entry {
    pub uuid: [u8; 16],
    pub group_id: u32,

    pub title: [u8; 64],
    pub username: [u8; 64],
    pub password: [u8; 64],
    pub times: Times,
    pub autotype: bool,
}

impl Entry {
    pub fn default_with_group_id(group_id: u32) -> Self {
        let uuid: [u8; 16] = 1u128.to_le_bytes();

        let mut title = [0u8; 64];
        title[..b"Google".len()].copy_from_slice(b"Google");

        let mut username = [0u8; 64];
        username[..b"carlos".len()].copy_from_slice(b"carlos");

        let mut password = [0u8; 64];
        password[..b"123456".len()].copy_from_slice(b"123456");

        let times = Times::zero();
        let autotype = true;

        Entry {
            uuid,
            group_id,
            title,
            username,
            password,
            times,
            autotype,
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let uuid: [u8; 16] = bytes[0..16].try_into().unwrap();
        let group_id = u32::from_le_bytes(bytes[16..20].try_into().unwrap());

        let title: [u8; 64] = bytes[20..84].try_into().unwrap();
        let username: [u8; 64] = bytes[84..148].try_into().unwrap();
        let password: [u8; 64] = bytes[148..212].try_into().unwrap();
        let times = Times::new_from_bytes(&bytes[212..232]);
        let autotype = bytes[232] != 0;

        Entry {
            uuid,
            group_id,
            title,
            username,
            password,
            times,
            autotype,
        }
    }

    pub fn to_bytes(&self) -> [u8; ENTRY_SIZE] {
        let mut bytes = [0u8; ENTRY_SIZE];

        bytes[0..16].copy_from_slice(&self.uuid);
        bytes[16..20].copy_from_slice(&self.group_id.to_le_bytes());

        bytes[20..84].copy_from_slice(&self.title);
        bytes[84..148].copy_from_slice(&self.username);
        bytes[148..212].copy_from_slice(&self.password);
        bytes[212..232].copy_from_slice(&self.times.to_bytes());
        bytes[232] = self.autotype as u8;

        bytes
    }
}
