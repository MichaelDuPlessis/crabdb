use core::error;
use int::Int;
use null::Null;
use std::fmt;

pub mod int;
pub mod null;

/// The data type used to store the key length
type KeyLen = u16;
/// The number of bytes the key length requires
const KEY_LEN_NUM_BYTES: usize = std::mem::size_of::<KeyLen>();

/// The value under which an object is stored in the database
#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Key(Vec<u8>);

impl Key {
    /// Create a new Key from raw bytes
    /// The bytes provides must have this format:
    /// | 2 bytes the length of the key (n) | n bytes the key itself |
    pub fn new(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        if bytes.len() < KEY_LEN_NUM_BYTES {
            // making sure there is enough data
            Err(ObjectError)
        } else {
            let mut buffer = [0; KEY_LEN_NUM_BYTES];
            buffer.copy_from_slice(&bytes[..KEY_LEN_NUM_BYTES]);
            let key_len = KeyLen::from_be_bytes(buffer) as usize;

            if key_len > 0 {
                let key = Vec::from(&bytes[KEY_LEN_NUM_BYTES..key_len + KEY_LEN_NUM_BYTES]);

                Ok((Self(key), &bytes[key_len + KEY_LEN_NUM_BYTES..]))
            } else {
                // Key len cannot be 0
                Err(ObjectError)
            }
        }
    }
}

/// Used to represent the type of the object
pub type TypeId = u8;
/// The amount of bytes TypeId requires
const TYPE_ID_NUM_BYTES: usize = std::mem::size_of::<TypeId>();

/// The type of errors that can occur when constructing an object
#[derive(Debug)]
pub struct ObjectError;

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the data provided for the Object is invalid")
    }
}

impl error::Error for ObjectError {}

/// The TypeId for Null
const NULL_TYPE_ID: u8 = 0;
/// The TypeId for Int
const INT_TYPE_ID: u8 = 1;

/// Represents an Object in the database
#[derive(Debug, Clone)]
pub enum Object {
    /// This represents no item
    Null(Null),
    /// A signed integer
    Int(Int),
}

impl Object {
    // Turn an Object into raw bytes
    pub fn serialize(&self) -> Vec<u8> {
        // TODO: A macro would be great here
        match self {
            Object::Null(null) => null.serialize(),
            Object::Int(int) => int.serialize(),
        }
    }

    /// Create an Object from raw bytes
    pub fn deserialize(bytes: &[u8]) -> Result<Self, ObjectError> {
        // First extract the TypeId
        let mut buffer = [0; TYPE_ID_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..TYPE_ID_NUM_BYTES]);
        let type_id = TypeId::from_be_bytes(buffer);

        let bytes = &bytes[TYPE_ID_NUM_BYTES..];

        // TODO: A macro would be great here
        match type_id {
            // Null
            NULL_TYPE_ID => Null::deserialize(bytes).map(|(object, _)| Self::Null(object)),
            // Int
            INT_TYPE_ID => Int::deserialize(bytes).map(|(object, _)| Self::Int(object)),
            // if there is no valid type then return an error
            _ => Err(ObjectError),
        }
    }
}

impl From<Option<Object>> for Object {
    fn from(value: Option<Object>) -> Self {
        match value {
            Some(object) => object,
            None => Self::Null(Null),
        }
    }
}
