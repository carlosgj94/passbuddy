use super::times::Times;

use defmt::Format;
use heapless::String;

type HStr<const N: usize> = String<N>;

#[derive(Clone, Format)]
pub struct AutoType {
    /// Whether autotype is enabled for this entry
    pub enabled: bool,
    /// Optional per-entry sequence; falls back to the database default when absent
    pub sequence: Option<HStr<64>>,
}

#[derive(Clone, Format)]
pub struct Entry {
    pub uuid: [u8; 16],
    pub group_id: u32,

    pub title: HStr<64>,
    pub username: HStr<64>,
    pub password: HStr<64>,
    pub url: HStr<128>,

    pub notes: HStr<256>,
    pub icon_id: u16,
    pub times: Times,
    pub autotype: Option<AutoType>,
}
