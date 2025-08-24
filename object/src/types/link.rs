use crate::{Key, ObjectError};

/// Represents a map (mapping of field names to objects) in the database
#[derive(Debug)]
pub struct Link;

impl Link {
    /// Validate link data and extract the consumed portion
    /// Link format: same as Key
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        Key::validate_and_extract(bytes)
    }
}
