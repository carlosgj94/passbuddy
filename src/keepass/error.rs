#[derive(Debug, defmt::Format)]
pub enum KDBError {
    /// The database is corrupted
    DatabaseIntegrityError,
    /// The item select wasn't found
    EntryNotFound,
}
