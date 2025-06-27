use crate::{Object, TypeId};

const TYPE_ID: TypeId = 0;

/// This represents a null object in the database
/// so just no value
#[derive(Debug)]
pub struct Null;

impl Object for Null {
    fn type_id(&self) -> TypeId {
        TYPE_ID
    }

    fn serialize(self) -> Vec<u8> {
        Vec::with_capacity(0)
    }
}
