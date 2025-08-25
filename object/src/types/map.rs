use crate::{Object, ObjectError, ObjectKind, slice_to_num};

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
pub struct Map(Box<[u8]>);

impl Map {
    /// Get the number of fields in the map
    pub fn num_fields(&self) -> FieldCount {
        slice_to_num!(FieldCount, &self.0[..FIELD_COUNT_NUM_BYTES])
    }

    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(object: Object) -> Self {
        Self(object.data)
    }

    /// Validate map data and extract the consumed portion
    /// Map format: | 2 bytes field count | field entries... |
    /// Field entry: | 2 bytes name length | name bytes | serialized object |
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < FIELD_COUNT_NUM_BYTES {
            return Err(ObjectError);
        }

        // Extract the field count
        let field_count = slice_to_num!(FieldCount, &bytes[..FIELD_COUNT_NUM_BYTES]) as usize;

        let mut remaining_bytes = &bytes[FIELD_COUNT_NUM_BYTES..];

        // Validate each field in the map
        for _ in 0..field_count {
            // Read field name length
            if remaining_bytes.len() < FIELD_NAME_LEN_NUM_BYTES {
                return Err(ObjectError);
            }

            let field_name_len =
                slice_to_num!(FieldNameLen, &remaining_bytes[..FIELD_NAME_LEN_NUM_BYTES]) as usize;
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

impl From<Map> for Object {
    fn from(value: Map) -> Self {
        Self {
            kind: ObjectKind::Map,
            data: value.0,
        }
    }
}

impl TryFrom<Object> for Map {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::Map {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}

pub struct MapIterator {
    bytes_consumed: usize,
    data: Box<[u8]>,
}

impl MapIterator {
    /// Create a new ListIterator
    fn new(list: Map) -> Self {
        Self {
            bytes_consumed: FIELD_COUNT_NUM_BYTES,
            data: list.0,
        }
    }
}

impl Iterator for MapIterator {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes_consumed == self.data.len() {
            None
        } else {
            let data = &self.data[self.bytes_consumed..];

            // This data should be in an existing map so it has to be valid
            // first extract the field name

            // TODO: This calls the normal deserialize method but a deserialize_unchecked may yield benefits
            let (object, remaining) = unsafe { Object::deserialize(&self.data).unwrap_unchecked() };
            self.bytes_consumed = data.len() - remaining.len();

            Some(object)
        }
    }
}

impl IntoIterator for Map {
    type Item = Object;

    type IntoIter = MapIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

// impl From<&[Object]> for List {
//     fn from(value: &[Object]) -> Self {
//         let len = value.len() as ListLen;

//         // TODO: Investigate if "with_capacity" is possible here
//         let mut data = Vec::new();
//         data.extend(len.to_be_bytes());
//         for object in value {
//             data.extend(object.data());
//         }

//         Self(data.into_boxed_slice())
//     }
// }
