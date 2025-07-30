use crate::ObjectError;

/// This represents a null object in the database
/// so just no value
#[derive(Debug)]
pub struct Null;

impl Null {
    /// Validate null data and extract the consumed portion
    /// Null objects have no data, so we consume nothing
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        // Null has no data, so we return empty slice and all remaining bytes
        Ok((&[], bytes))
    }
}
