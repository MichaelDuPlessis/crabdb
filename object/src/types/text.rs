use super::type_ids::{TEXT_TYPE_ID, TYPE_ID_NUM_BYTES};
use crate::{Object, ObjectError};

/// The data type used to store the length of Text in the payload
type TextLen = u16;
/// The nubmer of bytes used to reprsent the length of Text in the Payload
const TEXT_LEN_NUM_BYTES: usize = std::mem::size_of::<TextLen>();

/// Represents a piece of text in the database
/// The validity of the bytes that make up the text are not verified
#[derive(Debug, Clone)]
pub struct Text(Box<[u8]>);

impl Text {
    pub fn serialize(&self) -> Vec<u8> {
        // converting to bytes
        let bytes = &self.0;

        // provisioning size
        // must remember to include space for TypeId
        let mut data = Vec::with_capacity(TYPE_ID_NUM_BYTES + TEXT_LEN_NUM_BYTES + bytes.len());

        // building bytes
        data.push(TEXT_TYPE_ID);
        data.extend((bytes.len() as TextLen).to_be_bytes());
        data.extend(bytes);

        data
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        let bytes = bytes.as_ref();

        // Making sure that bytes is the exact right size for
        // the underlying type of Int
        if bytes.len() < TEXT_LEN_NUM_BYTES {
            Err(ObjectError)
        } else {
            // First extract the text length
            let mut buffer = [0; TEXT_LEN_NUM_BYTES];
            buffer.copy_from_slice(&bytes[..TEXT_LEN_NUM_BYTES]);
            let text_len = TextLen::from_be_bytes(buffer) as usize;

            let bytes = &bytes[TEXT_LEN_NUM_BYTES..];

            // if there is not enough bytes to match the text length
            if bytes.len() < text_len {
                Err(ObjectError)
            } else {
                let text = Box::from(&bytes[..text_len]);

                Ok((Self(text), &bytes[text_len..]))
            }
        }
    }
}

impl From<Text> for Object {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}
