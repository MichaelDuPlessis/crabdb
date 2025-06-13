use crate::{Object, ObjectError, RawObjectData, slice_to_array};
use logging::debug;

/// The number type to use of the Int data object
type IntType = isize;
/// The Int data type. It is internally reprsented as an isize.
#[derive(Debug, Clone)]
pub struct Int(IntType);

impl Int {
    /// Creates a new Int from an isize
    pub fn new(num: impl Into<IntType>) -> Self {
        Self(num.into())
    }
}

impl From<isize> for Int {
    fn from(value: isize) -> Self {
        Self::new(value)
    }
}

impl Object for Int {}

impl TryFrom<RawObjectData> for Int {
    type Error = ObjectError;

    fn try_from(value: RawObjectData) -> Result<Self, Self::Error> {
        let value = value.as_ref();

        // making sure there is exactly the correct amount of data
        if value.len() < std::mem::size_of::<IntType>() {
            debug!(
                "Data recieved is too short for Int. Len recieved: {}",
                value.len()
            );
            Err(ObjectError::MissingData)
        } else {
            Ok(Self::new(IntType::from_be_bytes(unsafe {
                slice_to_array(&value[..std::mem::size_of::<IntType>()])
            })))
        }
    }
}
