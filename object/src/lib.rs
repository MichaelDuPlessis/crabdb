use autocodec::Codec;

/// The value under which an object is stored in the database
#[derive(Debug, Hash, PartialEq, Eq, Clone, Codec)]
pub struct Key(#[codec(len = "u16", min_len = 1)] Box<[u8]>);

/// A field in a Map: field name (raw bytes with u16 length prefix) + value
#[derive(Debug, Clone, Codec)]
pub struct MapField {
    #[codec(len = "u16")]
    pub name: Box<[u8]>,
    pub value: Object,
}

/// Represents an Object in the database
#[derive(Debug, Clone, Codec)]
#[repr(u8)]
pub enum Object {
    /// Null - no value
    Null = 0,
    /// A signed 64-bit integer
    Int(i64) = 1,
    /// A text (string) object with u16 length prefix
    Text(#[codec(len = "u16")] Box<[u8]>) = 2,
    /// A list of objects with u16 length prefix
    List(#[codec(len = "u16")] Vec<Object>) = 3,
    /// A map of field names to objects with u16 length prefix
    Map(#[codec(len = "u16")] Vec<MapField>) = 4,
    /// A link to another object (same format as Key)
    Link(Key) = 5,
}

impl Object {
    /// Creates a null object
    pub fn null() -> Self {
        Self::Null
    }
}

impl From<Option<Object>> for Object {
    fn from(value: Option<Object>) -> Self {
        match value {
            Some(object) => object,
            None => Self::null(),
        }
    }
}

impl From<Key> for Object {
    fn from(key: Key) -> Self {
        Self::Link(key)
    }
}
