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
#[derive(Debug)]
pub struct Map;

impl Map {
    /// Validate map data and extract the consumed portion
    /// Map format: | 2 bytes field count | field entries... |
    /// Field entry: | 2 bytes name length | name bytes | serialized object |
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < FIELD_COUNT_NUM_BYTES {
            return Err(ObjectError);
        }

        // Extract the field count
        let mut buffer = [0; FIELD_COUNT_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..FIELD_COUNT_NUM_BYTES]);
        let field_count = FieldCount::from_be_bytes(buffer) as usize;

        let mut remaining_bytes = &bytes[FIELD_COUNT_NUM_BYTES..];

        // Validate each field in the map
        for _ in 0..field_count {
            // Read field name length
            if remaining_bytes.len() < FIELD_NAME_LEN_NUM_BYTES {
                return Err(ObjectError);
            }

            let mut buffer = [0; FIELD_NAME_LEN_NUM_BYTES];
            buffer.copy_from_slice(&remaining_bytes[..FIELD_NAME_LEN_NUM_BYTES]);
            let field_name_len = FieldNameLen::from_be_bytes(buffer) as usize;
            remaining_bytes = &remaining_bytes[FIELD_NAME_LEN_NUM_BYTES..];

            // Validate field name exists and is valid UTF-8
            if remaining_bytes.len() < field_name_len {
                return Err(ObjectError);
            }

            let field_name_bytes = &remaining_bytes[..field_name_len];
            if std::str::from_utf8(field_name_bytes).is_err() {
                return Err(ObjectError);
            }
            remaining_bytes = &remaining_bytes[field_name_len..];

            // Validate the object by deserializing it
            let (_, rest) = Object::deserialize(remaining_bytes)?;
            remaining_bytes = rest;
        }

        // Calculate how much we consumed
        let consumed_len = bytes.len() - remaining_bytes.len();
        let (consumed, remaining) = bytes.split_at(consumed_len);
        Ok((consumed, remaining))
    }
}
