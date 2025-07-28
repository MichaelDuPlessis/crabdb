use core::error;
use std::fmt;
use types::{
    int::Int,
    list::List,
    map::Map,
    null::Null,
    text::Text,
    type_ids::{self, TYPE_ID_NUM_BYTES, TypeId},
};

pub mod types;

/// The data type used to store the key length
type KeyLen = u16;
/// The number of bytes the key length requires
const KEY_LEN_NUM_BYTES: usize = std::mem::size_of::<KeyLen>();

/// The value under which an object is stored in the database
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Key(Box<[u8]>); // we don't care about the capcity

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
                let key = Box::from(&bytes[KEY_LEN_NUM_BYTES..key_len + KEY_LEN_NUM_BYTES]);

                Ok((Self(key), &bytes[key_len + KEY_LEN_NUM_BYTES..]))
            } else {
                // Key len cannot be 0
                Err(ObjectError)
            }
        }
    }

    /// Converts a key to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(KEY_LEN_NUM_BYTES + self.0.len());
        let mut buffer = [0; KEY_LEN_NUM_BYTES];
        buffer.copy_from_slice(&(self.0.len() as u16).to_be_bytes());
        bytes.extend_from_slice(&buffer);
        bytes.extend_from_slice(&self.0);
        bytes
    }
}

/// The type of errors that can occur when constructing an object
#[derive(Debug)]
pub struct ObjectError;

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the data provided for the Object is invalid")
    }
}

impl error::Error for ObjectError {}

/// Represents an Object in the database
#[derive(Debug, Clone)]
pub enum Object {
    /// This represents no item
    Null(Null),
    /// A signed integer
    Int(Int),
    /// A text (string) object
    Text(Text),
    /// A list of objects
    List(List),
    /// A map (mapping of string keys to objects)
    Map(Map),
}

impl Object {
    // Turn an Object into raw bytes
    pub fn serialize(&self) -> Vec<u8> {
        // TODO: A macro would be great here
        match self {
            Object::Null(null) => null.serialize(),
            Object::Int(int) => int.serialize(),
            Object::Text(text) => text.serialize(),
            Object::List(list) => list.serialize(),
            Object::Map(map) => map.serialize(),
        }
    }

    /// Create an Object from raw bytes
    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        // First extract the TypeId
        let mut buffer = [0; TYPE_ID_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..TYPE_ID_NUM_BYTES]);
        let type_id = TypeId::from_be_bytes(buffer);

        let bytes = &bytes[TYPE_ID_NUM_BYTES..];

        // TODO: A macro would be great here
        match type_id {
            // Null
            type_ids::NULL_TYPE_ID => {
                Null::deserialize(bytes).map(|(obj, bytes)| (obj.into(), bytes))
            }
            // Int
            type_ids::INT_TYPE_ID => {
                Int::deserialize(bytes).map(|(obj, bytes)| (obj.into(), bytes))
            }
            type_ids::TEXT_TYPE_ID => {
                Text::deserialize(bytes).map(|(obj, bytes)| (obj.into(), bytes))
            }
            type_ids::LIST_TYPE_ID => {
                List::deserialize(bytes).map(|(obj, bytes)| (obj.into(), bytes))
            }
            type_ids::MAP_TYPE_ID => {
                Map::deserialize(bytes).map(|(obj, bytes)| (obj.into(), bytes))
            }
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
