use crate::{ObjectError, object::Object, slice_to_array};
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

    /// Creates a Boxe dyn Object from RawObject data
    pub fn from_raw_object_data(
        object_data: Vec<u8>,
    ) -> Result<Box<dyn Object + Send + Sync>, <Self as TryFrom<Vec<u8>>>::Error> {
        Box::<Self>::try_from(object_data).map(|object| object as Box<dyn Object + Send + Sync>)
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Object for Text {
    fn boxed_clone(&self) -> Box<dyn Object + Send + Sync> {
        Box::new(self.clone())
    }

    fn into_raw(&self) -> Vec<u8> {
        let mut text = Vec::with_capacity(self.0.len() + TEXT_LEN_TYPE_NUM_BYTES);
        text.extend_from_slice(&(self.0.len() as TextLenType).to_be_bytes());
        text.extend_from_slice(&self.0.as_bytes());

        text
    }

    fn type_name(&self) -> &'static str {
        "text"
    }
}

impl TryFrom<Vec<u8>> for Text {
    type Error = ObjectError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
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

impl TryFrom<Vec<u8>> for Box<Text> {
    type Error = ObjectError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Text::try_from(value).map(|text| Box::new(text))
    }
}
