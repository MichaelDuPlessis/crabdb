use super::type_ids::{MAP_TYPE_ID, TYPE_ID_NUM_BYTES};
use crate::{Object, ObjectError};

/// The data type used to store the length of field name in the payload
type FieldNameLen = u16;
/// The number of bytes used to represent the length of field name in the payload
const FIELD_NAME_LEN_NUM_BYTES: usize = std::mem::size_of::<FieldNameLen>();

/// The data type used to store the number of fields in the map
type FieldCount = u16;
/// The number of bytes used to represent the field count in the payload
const FIELD_COUNT_NUM_BYTES: usize = std::mem::size_of::<FieldCount>();

/// Represents a map (mapping of field names to objects) in the database
#[derive(Debug, Clone)]
pub struct Map {
    fields: Box<[(Box<[u8]>, Object)]>,
}

impl Map {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized_fields = Vec::new();

        // Serialize each field (name + object)
        for (field_name, object) in &self.fields {
            // Field name length + field name + serialized object
            let field_name_len = field_name.len() as FieldNameLen;
            serialized_fields.extend(field_name_len.to_be_bytes());
            serialized_fields.extend(field_name.as_ref());
            serialized_fields.extend(object.serialize());
        }

        let mut bytes = Vec::with_capacity(
            TYPE_ID_NUM_BYTES + FIELD_COUNT_NUM_BYTES + serialized_fields.len()
        );
        
        bytes.extend(MAP_TYPE_ID.to_be_bytes());
        bytes.extend((self.fields.len() as FieldCount).to_be_bytes());
        bytes.extend(serialized_fields);

        bytes
    }

    pub fn deserialize(bytes: &[u8]) -> Result<(Self, &[u8]), ObjectError> {
        let bytes = bytes.as_ref();

        // Making sure there's enough bytes for the field count
        if bytes.len() < FIELD_COUNT_NUM_BYTES {
            return Err(ObjectError);
        }

        // First extract the field count
        let mut buffer = [0; FIELD_COUNT_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..FIELD_COUNT_NUM_BYTES]);
        let field_count = FieldCount::from_be_bytes(buffer) as usize;

        let mut remaining_bytes = &bytes[FIELD_COUNT_NUM_BYTES..];
        let mut fields = Vec::with_capacity(field_count);

        // Deserialize each field in the map
        for _ in 0..field_count {
            // Read field name length
            if remaining_bytes.len() < FIELD_NAME_LEN_NUM_BYTES {
                return Err(ObjectError);
            }
            
            let mut buffer = [0; FIELD_NAME_LEN_NUM_BYTES];
            buffer.copy_from_slice(&remaining_bytes[..FIELD_NAME_LEN_NUM_BYTES]);
            let field_name_len = FieldNameLen::from_be_bytes(buffer) as usize;
            remaining_bytes = &remaining_bytes[FIELD_NAME_LEN_NUM_BYTES..];

            // Read field name
            if remaining_bytes.len() < field_name_len {
                return Err(ObjectError);
            }
            
            let field_name = Box::from(&remaining_bytes[..field_name_len]);
            remaining_bytes = &remaining_bytes[field_name_len..];

            // Deserialize the object
            let (object, rest) = Object::deserialize(remaining_bytes)?;
            fields.push((field_name, object));
            remaining_bytes = rest;
        }

        Ok((
            Self {
                fields: fields.into_boxed_slice(),
            },
            remaining_bytes,
        ))
    }
}

impl From<Map> for Object {
    fn from(value: Map) -> Self {
        Self::Map(value)
    }
}
