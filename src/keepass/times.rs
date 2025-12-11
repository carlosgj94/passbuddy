use defmt::Format;

#[derive(Clone, Copy, Format)]
pub struct Times {
    pub created: u64, // FILETIME ticks
    pub modified: u64,
    pub accessed: u64,
    pub expires: u64, // 0 = never expires in KDB1
}
