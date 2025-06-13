use crate::{Deserialize, DeserializeError, Object, Serialize, SerializeError, slice_to_array};
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

impl Object for Text {}

impl Serialize<Vec<u8>> for Text {
    fn serialize(self) -> Result<Vec<u8>, SerializeError> {
        let mut text = Vec::with_capacity(self.0.len() + TEXT_LEN_TYPE_NUM_BYTES);
        text.extend_from_slice(&(self.0.len() as TextLenType).to_be_bytes());
        text.extend_from_slice(&self.0.as_bytes());

        Ok(text)
    }
}

impl Deserialize<&[u8]> for Text {
    fn deserialize(source: &[u8]) -> Result<Box<Self>, DeserializeError> {
        // making sure there is enough data
        if source.len() < TEXT_LEN_TYPE_NUM_BYTES {
            return Err(DeserializeError::MalformedData);
        }

        // extracting text length
        let text_len = TextLenType::from_be_bytes(unsafe {
            slice_to_array(&source[..TEXT_LEN_TYPE_NUM_BYTES])
        }) as usize;
        trace!("Text len: {text_len}");

        // making sure there is enough bytes left
        let source = &source[TEXT_LEN_TYPE_NUM_BYTES..];

        if source.len() < text_len {
            return Err(DeserializeError::MalformedData);
        }

        // try and convert byte slice to string
        let text = str::from_utf8(&source[..text_len])
            .map_err(|_| DeserializeError::MalformedData)?
            .to_owned();

        trace!("Text {text}");
        Ok(Box::new(Text::new(text)))
    }
}
