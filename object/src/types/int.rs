use crate::{Object, ObjectError, ObjectKind};

/// The internal type used to represent an int
type InternalInt = i64;
/// The number of bytes needed to represent an InternalInt
const INTERNAL_INT_SIZE: usize = std::mem::size_of::<InternalInt>();

/// The number data type that is stored in the database
/// it is backed by a signed 64 bit integer
#[derive(Debug)]
pub struct Int(Box<[u8]>);

impl Int {
    /// Get the stored number
    pub fn inner(&self) -> InternalInt {
        let mut num = [0; INTERNAL_INT_SIZE];
        num.copy_from_slice(&self.0[..INTERNAL_INT_SIZE]);

        InternalInt::from_be_bytes(num)
    }

    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(object: Object) -> Self {
        Self(object.data)
    }

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

impl From<Int> for Object {
    fn from(value: Int) -> Self {
        Self {
            kind: ObjectKind::Int,
            data: value.0,
        }
    }
}

impl TryFrom<Object> for Int {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::Int {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}
