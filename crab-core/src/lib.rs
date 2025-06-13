pub mod int;
pub mod text;

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

/// Anything that implements object is valid to store and retrieve from the database
pub trait Object: std::fmt::Debug + Serialize<Vec<u8>> + for<'a> Deserialize<&'a [u8]> {}
