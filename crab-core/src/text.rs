use crate::{Object, ObjectError, RawObjectData, slice_to_array};
use logging::trace;

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
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Object for Text {
    fn boxed_clone(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }
}

impl TryFrom<RawObjectData> for Text {
    type Error = ObjectError;

    fn try_from(value: RawObjectData) -> Result<Self, Self::Error> {
        let value = value.as_ref();

        // making sure there is enough data
        if value.len() < TEXT_LEN_TYPE_NUM_BYTES {
            return Err(ObjectError::MissingData);
        }

        // extracting text length
        let text_len = TextLenType::from_be_bytes(unsafe {
            slice_to_array(&value[..TEXT_LEN_TYPE_NUM_BYTES])
        }) as usize;
        trace!("Text len: {text_len}");

        // making sure there is enough bytes left
        let value = &value[TEXT_LEN_TYPE_NUM_BYTES..];

        if value.len() < text_len {
            return Err(ObjectError::MissingData);
        }

        // try and convert byte slice to string
        let text = str::from_utf8(&value[..text_len])
            .map_err(|_| ObjectError::MalformedData)?
            .to_owned();

        trace!("Text {text}");
        Ok(Text::new(text))
    }
}

impl TryFrom<RawObjectData> for Box<Text> {
    type Error = ObjectError;

    fn try_from(value: RawObjectData) -> Result<Self, Self::Error> {
        Text::try_from(value).map(|text| Box::new(text))
    }
}
