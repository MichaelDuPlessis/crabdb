use crate::{NULL_TYPE_ID, ObjectError};
/// This represents a null object in the database
/// so just no value
#[derive(Debug, Clone)]
pub struct Null;

impl Null {
    pub fn serialize(&self) -> Vec<u8> {
        vec![NULL_TYPE_ID]
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError>
    where
        Self: Sized,
    {
        Ok((Self, bytes))
    }
}
