use crate::ObjectError;

/// The internal type used to represent an int
type InternalInt = i64;
/// The number of bytes needed to represent an InternalInt
const INTERNAL_INT_SIZE: usize = std::mem::size_of::<InternalInt>();

/// The number data type that is stored in the database
/// it is backed by a signed 64 bit integer
#[derive(Debug, Clone)]
pub struct Int;

impl Int {
    /// Validate int data and extract the consumed portion
    /// Int objects are exactly 8 bytes (i64 in big-endian)
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < INTERNAL_INT_SIZE {
            Err(ObjectError)
        } else {
            let (int_bytes, remaining) = bytes.split_at(INTERNAL_INT_SIZE);
            Ok((int_bytes, remaining))
        }
    }
}
