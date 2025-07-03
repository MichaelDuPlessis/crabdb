use crate::ObjectError;
/// This represents a null object in the database
/// so just no value
#[derive(Debug)]
pub struct Null;

impl Null {
    pub fn serialize(&self) -> Vec<u8> {
        Vec::with_capacity(0)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError>
    where
        Self: Sized,
    {
        Ok((Self, bytes))
    }
}
