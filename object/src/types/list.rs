use super::type_ids::{LIST_TYPE_ID, TYPE_ID_NUM_BYTES};
use crate::{Object, ObjectError};

/// The data type used to store the length of List in the payload
type ListLen = u16;
/// The nubmer of bytes used to reprsent the length of List in the Payload
const LIST_LEN_NUM_BYTES: usize = std::mem::size_of::<ListLen>();

/// Represents a piece of text in the database
/// The validity of the bytes that make up the Objects in the list are not verified
#[derive(Debug, Clone)]
pub struct List {
    objects: Box<[Object]>,
}

impl List {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized_objects = Vec::with_capacity(self.objects.len());

        // serializeing objects
        for object in &self.objects {
            serialized_objects.push(object.serialize());
        }

        // flattening to byte array
        let serialized_objects: Vec<_> = serialized_objects.into_iter().flatten().collect();

        let mut bytes =
            Vec::with_capacity(TYPE_ID_NUM_BYTES + LIST_LEN_NUM_BYTES + serialized_objects.len());
        bytes.extend(LIST_TYPE_ID.to_be_bytes());
        bytes.extend((self.objects.len() as ListLen).to_be_bytes());
        bytes.extend(serialized_objects);

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        let bytes = bytes.as_ref();

        // Making sure there's enough bytes for the list length
        if bytes.len() < LIST_LEN_NUM_BYTES {
            return Err(ObjectError);
        }

        // First extract the list length
        let mut buffer = [0; LIST_LEN_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..LIST_LEN_NUM_BYTES]);
        let list_len = ListLen::from_be_bytes(buffer) as usize;

        let mut remaining_bytes = &bytes[LIST_LEN_NUM_BYTES..];
        let mut objects = Vec::with_capacity(list_len);

        // Deserialize each object in the list
        for _ in 0..list_len {
            let (object, rest) = Object::deserialize(remaining_bytes)?;
            objects.push(object);
            remaining_bytes = rest;
        }

        Ok((
            Self {
                objects: objects.into_boxed_slice(),
            },
            remaining_bytes,
        ))
    }
}

impl From<List> for Object {
    fn from(value: List) -> Self {
        Self::List(value)
    }
}
