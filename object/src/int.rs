use crate::{INT_TYPE_ID, ObjectError, TYPE_ID_NUM_BYTES};

/// The internal type used to represent an int
type InternalInt = i64;
/// The number of bytes needed to represent an InternalInt
const INTERNAL_INT_SIZE: usize = std::mem::size_of::<InternalInt>();

/// The number data type that is stored in the database
/// it is backed by a signed 64 bit integer
#[derive(Debug, Clone)]
pub struct Int(InternalInt);

impl Int {
    pub fn serialize(&self) -> Vec<u8> {
        // converting to bytes
        let bytes = self.0.to_be_bytes();

        // provisioning size
        // must remember to include space for TypeId
        let mut data = Vec::with_capacity(TYPE_ID_NUM_BYTES + bytes.len());

        // building bytes
        data.push(INT_TYPE_ID);
        data.extend(bytes);

        data
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError>
    where
        Self: Sized,
    {
        let bytes = bytes.as_ref();

        // Making sure that bytes is the exact right size for
        // the underlying type of Int
        if bytes.len() < INTERNAL_INT_SIZE {
            Err(ObjectError)
        } else {
            let mut buffer = [0; INTERNAL_INT_SIZE];
            buffer.copy_from_slice(&bytes[..INTERNAL_INT_SIZE]);

            let interal = InternalInt::from_be_bytes(buffer);

            Ok((Self(interal), &bytes[INTERNAL_INT_SIZE..]))
        }
    }
}
