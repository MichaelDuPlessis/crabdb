use crate::{
    Deserialize, DeserializeError, OBJECT_TYPE_NUM_BYTES, Object, Serialize, SerializeError,
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

    /// Creates a Int object
    pub fn new_object(num: impl Into<IntType>) -> Object {
        Object::Int(Self::new(num))
    }
}

impl From<isize> for Int {
    fn from(value: isize) -> Self {
        Self::new(value)
    }
}

impl Serialize<Vec<u8>> for Int {
    fn serialize(self) -> Result<Vec<u8>, SerializeError> {
        let mut int = [0; OBJECT_TYPE_NUM_BYTES + std::mem::size_of::<IntType>()];
        int[..OBJECT_TYPE_NUM_BYTES].copy_from_slice(&Object::INT_TAG.to_be_bytes());
        int[OBJECT_TYPE_NUM_BYTES..].copy_from_slice(&self.0.to_be_bytes());

        Ok(int.into())
    }
}

impl Deserialize<&[u8]> for Int {
    fn deserialize(source: &[u8]) -> Result<(Self, &[u8]), DeserializeError> {
        // making sure there is exactly the correct amount of data
        if source.len() < std::mem::size_of::<IntType>() {
            debug!(
                "Data recieved is too short for Int. Len recieved: {}",
                source.len()
            );
            Err(DeserializeError::MalformedData)
        } else {
            Ok((
                Self::new(IntType::from_be_bytes(unsafe {
                    slice_to_array(&source[..std::mem::size_of::<IntType>()])
                })),
                &source[..std::mem::size_of::<IntType>()],
            ))
        }
    }
}
