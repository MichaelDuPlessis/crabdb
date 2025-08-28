use crate::{Object, ObjectError, ObjectKind};

/// The data type used to store the length of Text in the payload
type TextLen = u16;
/// The number of bytes used to represent the length of Text in the Payload
const TEXT_LEN_NUM_BYTES: usize = std::mem::size_of::<TextLen>();

/// Represents a piece of text in the database
#[derive(Debug)]
pub struct Text(Box<[u8]>);

impl Text {
    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(object: Object) -> Self {
        Self(object.data)
    }

    /// Validate text data and extract the consumed portion
    /// Text format: | 2 bytes length | n bytes UTF-8 data |
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < TEXT_LEN_NUM_BYTES {
            return Err(ObjectError);
        }

        // Extract the text length
        let mut buffer = [0; TEXT_LEN_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..TEXT_LEN_NUM_BYTES]);
        let text_len = TextLen::from_be_bytes(buffer) as usize;

        let total_size = TEXT_LEN_NUM_BYTES + text_len;
        if bytes.len() < total_size {
            return Err(ObjectError);
        }

        // Validate UTF-8
        let text_bytes = &bytes[TEXT_LEN_NUM_BYTES..total_size];
        if std::str::from_utf8(text_bytes).is_err() {
            return Err(ObjectError);
        }

        let (consumed, remaining) = bytes.split_at(total_size);
        Ok((consumed, remaining))
    }
}

impl From<Text> for Object {
    fn from(value: Text) -> Self {
        Self {
            kind: ObjectKind::Text,
            data: value.0,
        }
    }
}

impl TryFrom<Object> for Text {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::Text {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}
