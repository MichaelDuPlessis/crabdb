use logging::trace;
use object::ObjectError;

pub mod factory;
pub mod object;
pub mod types;

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

/// Extracts the key from a byte slice and returns what is remaining in the slice
// TODO: Should this be an associated function?
pub fn extract_key(slice: &[u8]) -> Result<(Key, &[u8]), ObjectError> {
    trace!("Deserializing key");

    // first check if the size is large enough
    if slice.len() < KEY_LEN_TYPE_NUM_BYTES {
        return Err(ObjectError::MissingData);
    }

    // extracting text length
    let key_len =
        KeyLenType::from_be_bytes(unsafe { slice_to_array(&slice[..KEY_LEN_TYPE_NUM_BYTES]) })
            as usize;
    trace!("Key len: {key_len}");

    // making sure there is enough bytes left
    let slice = &slice[KEY_LEN_TYPE_NUM_BYTES..];

    if slice.len() < key_len {
        return Err(ObjectError::MissingData);
    }

    // try and convert byte slice to string
    let key = str::from_utf8(&slice[..key_len])
        .map_err(|_| ObjectError::MalformedData)?
        .to_owned();

    trace!("Extracted key: {key}");

    Ok((Key::new(key), &slice[key_len..]))
}
