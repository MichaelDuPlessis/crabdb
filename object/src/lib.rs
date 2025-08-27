use crate::types::link::Link;
use core::error;
use logging::trace;
use std::fmt;
use types::{int::Int, list::List, map::Map, null::Null, text::Text};

pub mod types;

/// Convert a slice to a number
#[macro_export]
macro_rules! slice_to_num {
    ($t:ty, $bytes:expr) => {{
        let mut arr = [0u8; ::std::mem::size_of::<$t>()];
        arr.copy_from_slice($bytes);
        <$t>::from_be_bytes(arr)
    }};
}

/// The data type used to store the key length
type KeyLen = u16;
/// The number of bytes the key length requires
const KEY_LEN_NUM_BYTES: usize = std::mem::size_of::<KeyLen>();

/// The value under which an object is stored in the database
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
// TODO: Currently key stores its length as well which seems a bit redundent but is used for the Link type
// this should possibly be changed and new intermediary type created since now an extra 2 bytes per key is wasted
pub struct Key(Box<[u8]>); // we don't care about the capcity

impl Key {
    /// Create a new Key from raw bytes
    pub fn new(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        let (key, rest) = Self::validate_and_extract(bytes)?;

        Ok((Self(Box::from(key)), rest))
    }

    /// Creates a new key from bytes without verification
    pub unsafe fn new_unchecked(bytes: &[u8]) -> Self {
        Self(Box::from(bytes))
    }

    /// Validate link data and extract the consumed portion
    /// Link format: | 2 bytes the length of the key (n) | n bytes the key itself |
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < KEY_LEN_NUM_BYTES {
            // making sure there is enough data
            Err(ObjectError)
        } else {
            let key_len = slice_to_num!(KeyLen, &bytes[..KEY_LEN_NUM_BYTES]) as usize;

            if key_len > 0 {
                let key = &bytes[..key_len + KEY_LEN_NUM_BYTES];

                Ok((key, &bytes[key_len + KEY_LEN_NUM_BYTES..]))
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

/// The types of Objects that can be stored in the database
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ObjectKind {
    /// This represents no item
    Null,
    /// A signed integer
    Int,
    /// A text (string) object
    Text,
    /// A list of objects
    List,
    /// A map (mapping of string keys to objects)
    Map,
    /// A link to another object in the database
    Link,
}

/// The number of bytes that the ObjectKind takes
const OBJECT_KIND_NUM_BYTES: usize = std::mem::size_of::<ObjectKind>();

impl TryFrom<u8> for ObjectKind {
    // TODO: spruce up object error so that it is more descriptive
    type Error = ObjectError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            val if val == Self::Null as u8 => Self::Null,
            val if val == Self::Int as u8 => Self::Int,
            val if val == Self::Text as u8 => Self::Text,
            val if val == Self::List as u8 => Self::List,
            val if val == Self::Map as u8 => Self::Map,
            val if val == Self::Link as u8 => Self::Link,
            _ => return Err(ObjectError),
        })
    }
}

/// Represents an Object in the database
#[derive(Debug, Clone)]
pub struct Object {
    /// The kind of object stored in the database
    kind: ObjectKind,
    /// The raw data of the object
    data: Box<[u8]>,
}

impl Object {
    /// Create a new Object from an ObjectKind and the raw data. The raw data is not checked so this can lead to UB.
    pub unsafe fn new_unchecked(kind: ObjectKind, data: Box<[u8]>) -> Self {
        Self { kind, data }
    }

    /// Creates a null object
    pub fn null() -> Self {
        Self {
            kind: ObjectKind::Null,
            data: Box::new([]),
        }
    }

    /// Get an Object's kind
    pub fn kind(&self) -> ObjectKind {
        self.kind
    }

    /// Get the raw data of the object
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Turn an Object into raw bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(std::mem::size_of::<ObjectKind>() + self.data.len());
        bytes.push(self.kind as u8);
        bytes.extend(&self.data);

        bytes
    }

    /// Create an Object from raw bytes
    // TODO: is deserialize the best method name
    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        if bytes.len() < OBJECT_KIND_NUM_BYTES {
            return Err(ObjectError);
        }

        // First extract the TypeId
        let type_id = bytes[0];
        let kind = ObjectKind::try_from(type_id)?;
        let data_bytes = &bytes[OBJECT_KIND_NUM_BYTES..];

        // Validate the data and determine how much we consumed
        let (data, remaining) = match kind {
            ObjectKind::Null => Null::validate_and_extract(data_bytes)?,
            ObjectKind::Int => Int::validate_and_extract(data_bytes)?,
            ObjectKind::Text => Text::validate_and_extract(data_bytes)?,
            ObjectKind::List => List::validate_and_extract(data_bytes)?,
            ObjectKind::Map => Map::validate_and_extract(data_bytes)?,
            ObjectKind::Link => Link::validate_and_extract(data_bytes)?,
        };

        Ok((
            Self {
                kind,
                data: Box::from(data),
            },
            remaining,
        ))
    }
}

impl From<Option<Object>> for Object {
    fn from(value: Option<Object>) -> Self {
        match value {
            Some(object) => object,
            None => Self::null(),
        }
    }
}
