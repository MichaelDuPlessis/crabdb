use crate::{Key, Object, ObjectError, ObjectKind};

/// Represents a map (mapping of field names to objects) in the database
#[derive(Debug)]
pub struct Link(Box<[u8]>);

impl Link {
    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(object: Object) -> Self {
        Self(object.data)
    }

    /// Validate link data and extract the consumed portion
    /// Link format: same as Key
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        Key::validate_and_extract(bytes)
    }
}

impl From<Link> for Object {
    fn from(value: Link) -> Self {
        Self {
            kind: ObjectKind::Int,
            data: value.0,
        }
    }
}

impl TryFrom<Object> for Link {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::Link {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}
