/// Errors that can occur when deseriarlizing
pub enum DeserializeError {
    /// The type specified is invalid
    InvalidType,
    /// The data provided for the type is invalid
    MalformedData,
}

/// Implementations of this trait must implement a way to convert some type to another type
pub trait Deserialize<T>: Sized {
    fn deserialize(source: T) -> Result<Self, DeserializeError>;
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

/// The number type to use of the Int data object
type IntType = isize;
/// The Int data type. It is internally reprsented as an isize.
#[derive(Debug)]
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

/// The number type that is used to determine the length of the text data type
type TextLenType = u16;
/// The number of bytes used to store the length of the text data type
const TEXT_LEN_TYPE_NUM_BYTES: usize = std::mem::size_of::<TextLenType>();

/// The Text data type. It is internally reprsented as an String.
#[derive(Debug)]
pub struct Text(String);

impl Text {
    /// Creates a new Text from a String
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    /// Creates a Text object
    pub fn new_object(text: impl Into<String>) -> Object {
        Object::Text(Self::new(text))
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// The available data types for the database
#[derive(Debug)]
pub enum Object {
    Int(Int),
    Text(Text),
    // Struct,
    // List,
}

impl Object {
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

impl Deserialize<&[u8]> for Object {
    fn deserialize(source: &[u8]) -> Result<Self, DeserializeError> {
        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        let object_type =
            ObjectType::from_be_bytes(unsafe { slice_to_array(&source[..OBJECT_TYPE_NUM_BYTES]) });
        let source = &source[OBJECT_TYPE_NUM_BYTES..];

        match object_type {
            // Int object
            0 => {
                // making sure there is exactly the correct amount of data
                if source.len() != std::mem::size_of::<IntType>() {
                    Err(DeserializeError::MalformedData)
                } else {
                    Ok(Self::new_int(IntType::from_be_bytes(unsafe {
                        slice_to_array(source)
                    })))
                }
            }
            // Text type
            1 => {
                // making sure there is enough data
                if source.len() < TEXT_LEN_TYPE_NUM_BYTES {
                    return Err(DeserializeError::MalformedData);
                }

                // extracting text length
                let text_len =
                    TextLenType::from_be_bytes(unsafe { slice_to_array(source) }) as usize;
                // making sure there is enough bytes left
                let source = &source[TEXT_LEN_TYPE_NUM_BYTES..];

                if source.len() != text_len {
                    return Err(DeserializeError::MalformedData);
                }

                // try and convert byte slice to string
                let text = str::from_utf8(source)
                    .map_err(|_| DeserializeError::MalformedData)?
                    .to_owned();
                Ok(Object::new_text(text))
            }
            _ => return Err(DeserializeError::InvalidType),
        }
    }
}

impl Deserialize<&[u8]> for Key {
    fn deserialize(source: &[u8]) -> Result<Self, DeserializeError> {
        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        // extracting text length
        let key_len = KeyLenType::from_be_bytes(unsafe { slice_to_array(source) }) as usize;
        // making sure there is enough bytes left
        let source = &source[TEXT_LEN_TYPE_NUM_BYTES..];

        if source.len() != key_len {
            return Err(DeserializeError::MalformedData);
        }

        // try and convert byte slice to string
        let key = str::from_utf8(source)
            .map_err(|_| DeserializeError::MalformedData)?
            .to_owned();

        Ok(Key::new(key))
    }
}
