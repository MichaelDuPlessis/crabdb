use crate::{ObjectError, object::Object, slice_to_array};
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
        object_data: Vec<u8>,
    ) -> Result<Box<dyn Object + Send + Sync>, <Self as TryFrom<Vec<u8>>>::Error> {
        Box::<Self>::try_from(object_data).map(|object| object as Box<dyn Object + Send + Sync>)
    }
}

impl From<isize> for Int {
    fn from(value: isize) -> Self {
        Self::new(value)
    }
}

impl Object for Int {
    fn boxed_clone(&self) -> Box<dyn Object + Send + Sync> {
        Box::new(self.clone())
    }

    fn into_raw(&self) -> Vec<u8> {
        self.0.to_be_bytes().into()
    }

    fn type_name(&self) -> &'static str {
        "int"
    }
}

impl TryFrom<Vec<u8>> for Int {
    type Error = ObjectError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
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

impl TryFrom<Vec<u8>> for Box<Int> {
    type Error = ObjectError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Int::try_from(value).map(|int| Box::new(int))
    }
}
