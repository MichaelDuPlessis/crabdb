use crate::{Object, ObjectError, ObjectKind};

/// The data type used to store the length of List in the payload
type ListLen = u16;
/// The number of bytes used to represent the length of List in the Payload
const LIST_LEN_NUM_BYTES: usize = std::mem::size_of::<ListLen>();

/// Represents a list in the database
#[derive(Debug)]
pub struct List(Box<[u8]>);

impl List {
    /// Get the Len of the list
    pub fn len(&self) -> ListLen {
        let mut len = [0; LIST_LEN_NUM_BYTES];
        len.copy_from_slice(&self.0[..LIST_LEN_NUM_BYTES]);

        ListLen::from_be_bytes(len)
    }

    /// Create an Int from an Object without verifying if it is valid (this method does not check the object_kind field)
    pub unsafe fn from_object_unchecked(object: Object) -> Self {
        Self(object.data)
    }

    /// Validate list data and extract the consumed portion
    /// List format: | 2 bytes count | serialized objects... |
    pub fn validate_and_extract(bytes: &[u8]) -> Result<(&[u8], &[u8]), ObjectError> {
        if bytes.len() < LIST_LEN_NUM_BYTES {
            return Err(ObjectError);
        }

        // Extract the list length
        let mut buffer = [0; LIST_LEN_NUM_BYTES];
        buffer.copy_from_slice(&bytes[..LIST_LEN_NUM_BYTES]);
        let list_len = ListLen::from_be_bytes(buffer) as usize;

        let mut remaining_bytes = &bytes[LIST_LEN_NUM_BYTES..];

        // Validate each object in the list by deserializing them
        for _ in 0..list_len {
            let (_, rest) = Object::deserialize(remaining_bytes)?;
            remaining_bytes = rest;
        }

        // Calculate how much we consumed
        let consumed_len = bytes.len() - remaining_bytes.len();
        let (consumed, remaining) = bytes.split_at(consumed_len);
        Ok((consumed, remaining))
    }
}

impl From<List> for Object {
    fn from(value: List) -> Self {
        Self {
            kind: ObjectKind::List,
            data: value.0,
        }
    }
}

impl TryFrom<Object> for List {
    type Error = ObjectError;

    fn try_from(value: Object) -> Result<Self, Self::Error> {
        if value.kind() == ObjectKind::List {
            Ok(unsafe { Self::from_object_unchecked(value) })
        } else {
            Err(ObjectError)
        }
    }
}

pub struct ListIterator {
    bytes_consumed: usize,
    data: Box<[u8]>,
}

impl ListIterator {
    /// Create a new ListIterator
    fn new(list: List) -> Self {
        Self {
            bytes_consumed: LIST_LEN_NUM_BYTES,
            data: list.0,
        }
    }
}

impl Iterator for ListIterator {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes_consumed == self.data.len() {
            None
        } else {
            let data = &self.data[self.bytes_consumed..];

            // This data should be in an existing list so it has to be valid
            // TODO: This calls the normal deserialize method but a deserialize_unchecked may yield benefits
            let (object, remaining) = unsafe { Object::deserialize(&self.data).unwrap_unchecked() };
            self.bytes_consumed = data.len() - remaining.len();

            Some(object)
        }
    }
}

impl IntoIterator for List {
    type Item = Object;

    type IntoIter = ListIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl From<&[Object]> for List {
    fn from(value: &[Object]) -> Self {
        let len = value.len() as ListLen;

        // TODO: Investigate if "with_capacity" is possible here
        let mut data = Vec::new();
        data.extend(len.to_be_bytes());
        for object in value {
            data.extend(object.data());
        }

        Self(data.into_boxed_slice())
    }
}
