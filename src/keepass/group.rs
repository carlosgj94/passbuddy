use super::times::Times;
use defmt::Format;
use heapless::String;

type HStr<const N: usize> = String<N>;

#[derive(Clone, Format)]
pub struct Group {
    /// The unique identifier of the group
    pub group_id: u32,

    /// The name of the group
    pub name: HStr<64>,

    /// ID of the group's icon
    pub icon_id: Option<u32>,

    /// Level of the group in the hierarchy
    pub level: u16,

    /// The list of time fields for this group
    pub times: Times,

    /// Flags for the group
    pub flags: Option<u32>,
}
