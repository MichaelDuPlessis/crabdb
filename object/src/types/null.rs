use crate::{Object, ObjectError, types::type_ids::NULL_TYPE_ID};
/// This represents a null object in the database
/// so just no value
#[derive(Debug, Clone)]
pub struct Null;

impl Null {
    pub fn serialize(&self) -> Vec<u8> {
        NULL_TYPE_ID.to_be_bytes().into()
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        Ok((Self, bytes))
    }
}

impl From<Null> for Object {
    fn from(value: Null) -> Self {
        Self::Null(value)
    }
}
