use crate::{
    ObjectError,
    object::{Object, RawObjectData},
    slice_to_array,
};
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

    /// Creates a Boxe dyn Object from RawObject data
    pub fn from_raw_object_data(
        object_data: RawObjectData,
    ) -> Result<Box<dyn Object>, <Self as TryFrom<RawObjectData>>::Error> {
        Box::<Self>::try_from(object_data).map(|object| object as Box<dyn Object>)
    }
}

impl From<isize> for Int {
    fn from(value: isize) -> Self {
        Self::new(value)
    }
}

impl Object for Int {
    fn boxed_clone(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn into_raw_object_data(&self) -> RawObjectData {
        RawObjectData::new(self.0.to_be_bytes())
    }
}

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

impl TryFrom<RawObjectData> for Box<Int> {
    type Error = ObjectError;

    fn try_from(value: RawObjectData) -> Result<Self, Self::Error> {
        Int::try_from(value).map(|int| Box::new(int))
    }
}
