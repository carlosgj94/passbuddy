use super::times::Times;

use defmt::Format;

// uuid = 16; group_id = 4; title = 64; username = 64; password = 64; url = 128;
// icon_id = 8; times = 20; autotype = 1; padding = 3;
pub const ENTRY_SIZE: usize = 16 + 4 + 64 + 64 + 64 + 128 + 8 + 20 + 1 + 3; // 372

#[derive(Clone, Copy, Format)]
pub struct Entry {
    pub uuid: [u8; 16],
    pub group_id: u32,

    pub title: [u8; 64],
    pub username: [u8; 64],
    pub password: [u8; 64],
    pub url: [u8; 128],

    pub icon_id: Option<u32>,
    pub times: Times,
    pub autotype: bool,
}

impl Entry {
    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let uuid: [u8; 16] = bytes[0..16].try_into().unwrap();
        let group_id = u32::from_le_bytes(bytes[16..20].try_into().unwrap());

        let title: [u8; 64] = bytes[20..84].try_into().unwrap();
        let username: [u8; 64] = bytes[84..148].try_into().unwrap();
        let password: [u8; 64] = bytes[148..212].try_into().unwrap();
        let url: [u8; 128] = bytes[212..340].try_into().unwrap();

        let icon_id_raw = u32::from_le_bytes(bytes[340..344].try_into().unwrap());
        let icon_id_present = u32::from_le_bytes(bytes[344..348].try_into().unwrap());
        let icon_id = if icon_id_present == 0 {
            None
        } else {
            Some(icon_id_raw)
        };

        let times = Times::new_from_bytes(&bytes[348..368]);
        let autotype = bytes[368] != 0;

        Entry {
            uuid,
            group_id,
            title,
            username,
            password,
            url,
            icon_id,
            times,
            autotype,
        }
    }
}
