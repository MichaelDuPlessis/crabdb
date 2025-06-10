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

/// The number type that is used to determine the length of the text data type
type TextLenType = u16;
/// The number of bytes used to store the length of the text data type
const TEXT_LEN_TYPE_NUM_BYTES: usize = std::mem::size_of::<TextLenType>();

/// The Text data type. It is internally reprsented as an String.
#[derive(Debug, Clone)]
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

impl Serialize<Vec<u8>> for Text {
    fn serialize(self) -> Result<Vec<u8>, SerializeError> {
        let mut text = Vec::with_capacity(OBJECT_TYPE_NUM_BYTES + self.0.len());
        text[..OBJECT_TYPE_NUM_BYTES].copy_from_slice(&Object::TEXT_TAG.to_be_bytes());
        text[OBJECT_TYPE_NUM_BYTES..].copy_from_slice(&self.0.as_bytes());

        Ok(text)
    }
}

impl Deserialize<&[u8]> for Text {
    fn deserialize(source: &[u8]) -> Result<(Self, &[u8]), DeserializeError> {
        // making sure there is enough data
        if source.len() < TEXT_LEN_TYPE_NUM_BYTES {
            return Err(DeserializeError::MalformedData);
        }

        // extracting text length
        let text_len = TextLenType::from_be_bytes(unsafe { slice_to_array(source) }) as usize;
        // making sure there is enough bytes left
        let source = &source[TEXT_LEN_TYPE_NUM_BYTES..];

        if source.len() < text_len {
            return Err(DeserializeError::MalformedData);
        }

        // try and convert byte slice to string
        let text = str::from_utf8(&source[..text_len])
            .map_err(|_| DeserializeError::MalformedData)?
            .to_owned();
        Ok((Text::new(text), &source[text_len..]))
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
        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        let object_type =
            ObjectType::from_be_bytes(unsafe { slice_to_array(&source[..OBJECT_TYPE_NUM_BYTES]) });
        let source = &source[OBJECT_TYPE_NUM_BYTES..];

        match object_type {
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
        // first check if the size is large enough
        if source.len() < 2 {
            return Err(DeserializeError::MalformedData);
        }

        // extracting text length
        let key_len = KeyLenType::from_be_bytes(unsafe { slice_to_array(source) }) as usize;
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
