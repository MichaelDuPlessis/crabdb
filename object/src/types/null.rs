use crate::{Object, ObjectError, ObjectKind};

/// This represents a null object in the database
/// so just no value
#[derive(Debug)]
pub struct Null;

impl Null {
    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(_: Object) -> Self {
        Self
    }

    /// Validate null data and extract the consumed portion
    /// Null objects have no data, so we consume nothing
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        // Null has no data, so we return empty slice and all remaining bytes
        Ok((&[], bytes))
    }
}

impl From<Null> for Object {
    fn from(_: Null) -> Self {
        Self {
            kind: crate::ObjectKind::Null,
            data: Box::new([]),
        }
    }
}

impl TryFrom<Object> for Null {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::Null {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}
