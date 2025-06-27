use crate::{Object, TypeId};

const TYPE_ID: TypeId = 1;

/// The number data type that is stored in the database
/// it is backed by a signed 64 bit integer
#[derive(Debug)]
pub struct Int(i64);

impl Object for Int {
    fn type_id(&self) -> TypeId {
        TYPE_ID
    }

    fn serialize(self) -> Vec<u8> {
        self.0.to_be_bytes().into()
    }
}
