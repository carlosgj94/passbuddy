use defmt::Format;

/// KeePass v1 stores timestamps as a packed 5-byte little-endian value (40 bits).
///
/// Layout (least-significant bits first):
/// - bits  0..= 5: second (0..=59)
/// - bits  6..=11: minute (0..=59)
/// - bits 12..=16: hour   (0..=23)
/// - bits 17..=21: day    (1..=31)
/// - bits 22..=25: month  (1..=12)
/// - bits 26..=39: year   (e.g. 2025)
#[derive(Clone, Copy, PartialEq, Eq, Format, Debug)]
pub struct KdbTime {
    raw: [u8; 5],
}

impl KdbTime {
    pub const NEVER: Self = Self { raw: [0; 5] };

    pub const fn from_raw(raw: [u8; 5]) -> Self {
        Self { raw }
    }

    pub const fn raw(&self) -> &[u8; 5] {
        &self.raw
    }
}

#[derive(Clone, Copy, Format, Debug)]
pub struct Times {
    pub created: KdbTime,
    pub modified: KdbTime,
    pub accessed: KdbTime,
    /// All-zero packed time means "never expires" in KeePass v1.
    pub expires: KdbTime,
}

impl Times {
    pub fn zero() -> Self {
        Self {
            created: KdbTime::NEVER,
            modified: KdbTime::NEVER,
            accessed: KdbTime::NEVER,
            expires: KdbTime::NEVER,
        }
    }

    pub fn new_from_bytes(bytes: &[u8]) -> Self {
        let created = KdbTime::from_raw(bytes[0..5].try_into().unwrap());
        let modified = KdbTime::from_raw(bytes[5..10].try_into().unwrap());
        let accessed = KdbTime::from_raw(bytes[10..15].try_into().unwrap());
        let expires = KdbTime::from_raw(bytes[15..20].try_into().unwrap());
        Self {
            created,
            modified,
            accessed,
            expires,
        }
    }

    pub fn to_bytes(&self) -> [u8; 20] {
        let mut bytes = [0u8; 20];
        bytes[0..5].copy_from_slice(self.created.raw());
        bytes[5..10].copy_from_slice(self.modified.raw());
        bytes[10..15].copy_from_slice(self.accessed.raw());
        bytes[15..20].copy_from_slice(self.expires.raw());
        bytes
    }
}
