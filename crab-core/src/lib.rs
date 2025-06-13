use int::Int;
use logging::trace;
use text::Text;

mod int;
mod text;

/// Errors that can occur when deseriarlizing
pub enum DeserializeError {
    /// The type specified is invalid
    InvalidType,
    /// The data provided for the type is invalid
    MalformedData,
}

/// Implementations of this trait must implement a way to convert some type to another type
pub trait Deserialize<T>: Sized {
    /// Converts from one type to another and returns that type along with what remains from the previous type
    fn deserialize(source: T) -> Result<(Self, T), DeserializeError>;
}

/// Errors that can occur when seriarlizing
pub enum SerializeError {}

/// Implementations of this tait must implement a way to serialize the type
pub trait Serialize<T> {
    /// Converts from a concrete type to another type that can be stored or sent
    fn serialize(self) -> Result<T, SerializeError>;
}

/// The data type of the number used to store the data type
type ObjectType = u8;
/// The number of bytes used to represent the data type
const OBJECT_TYPE_NUM_BYTES: usize = std::mem::size_of::<ObjectType>();

/// Converts a slice to a fixed size array unsafely
pub unsafe fn slice_to_array<T, const S: usize>(slice: &[T]) -> [T; S]
where
    [T; S]: for<'a> TryFrom<&'a [T]>,
{
    unsafe { slice.try_into().unwrap_unchecked() }
}

/// The number type that is used to determine the length of the text data type
type KeyLenType = u16;
/// The number of bytes used to store the length of the text data type
const KEY_LEN_TYPE_NUM_BYTES: usize = std::mem::size_of::<KeyLenType>();

/// What items are stored under in the database
// TODO: I don't care about the capacity of the string so maybe change to a len and u8 slice instead
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Key(String);

impl Key {
    /// Create a new key from a type that can be converted into a String
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl<T> From<T> for Key
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// The available data types for the database
#[derive(Debug, Clone)]
pub enum Object {
    /// Nothing
    Null,
    /// A number
    Int(Int),
    /// A string
    Text(Text),
    // Struct,
    // List,
}

impl Object {
    /// The value associates with nothing
    pub const NULL_TAG: u8 = 0;

    /// The value associated with an Int
    pub const INT_TAG: u8 = 1;

    /// The value associated with an Int
    pub const TEXT_TAG: u8 = 2;

    /// Creates a new Int type
    pub fn new_int<T>(num: T) -> Self
    where
        T: Into<Int>,
    {
        Self::Int(num.into())
    }

    /// Creates a new Text type
    pub fn new_text<T>(text: T) -> Self
    where
        T: Into<Text>,
    {
        Self::Text(text.into())
    }
}

impl Serialize<Vec<u8>> for Object {
    fn serialize(self) -> Result<Vec<u8>, SerializeError> {
        match self {
            Object::Null => Ok(vec![Object::NULL_TAG]),
            Object::Int(int) => int.serialize(),
            Object::Text(text) => text.serialize(),
        }
    }
}

impl Deserialize<&[u8]> for Object {
    fn deserialize(source: &[u8]) -> Result<(Self, &[u8]), DeserializeError> {
        trace!("Deserializing object: {source:?}");

        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        let object_type =
            ObjectType::from_be_bytes(unsafe { slice_to_array(&source[..OBJECT_TYPE_NUM_BYTES]) });
        let source = &source[OBJECT_TYPE_NUM_BYTES..];
        trace!("Object type: {object_type:?}");

        match object_type {
            Self::NULL_TAG => Ok((Self::Null, source)),
            // Int object
            Self::INT_TAG => Int::deserialize(source).map(|(int, rest)| (Self::new_int(int), rest)),
            // Text type
            Self::TEXT_TAG => {
                Text::deserialize(source).map(|(text, rest)| (Self::new_text(text), rest))
            }
            _ => return Err(DeserializeError::InvalidType),
        }
    }
}

impl Deserialize<&[u8]> for Key {
    fn deserialize(source: &[u8]) -> Result<(Self, &[u8]), DeserializeError> {
        trace!("Deserializing key");

        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        // extracting text length
        let key_len =
            KeyLenType::from_be_bytes(unsafe { slice_to_array(&source[..KEY_LEN_TYPE_NUM_BYTES]) })
                as usize;
        trace!("Key len: {key_len}");

        // making sure there is enough bytes left
        let source = &source[KEY_LEN_TYPE_NUM_BYTES..];

        if source.len() < key_len {
            return Err(DeserializeError::MalformedData);
        }

        // try and convert byte slice to string
        let key = str::from_utf8(&source[..key_len])
            .map_err(|_| DeserializeError::MalformedData)?
            .to_owned();

        Ok((Key::new(key), &source[key_len..]))
    }
}
