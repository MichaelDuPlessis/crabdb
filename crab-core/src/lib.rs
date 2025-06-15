use logging::trace;
use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

pub mod int;
pub mod null;
pub mod text;

/// The kinds of errors that can occur around objects
pub enum ObjectError {
    /// There is not enough data to build a object
    MissingData,
    /// The data provided is malformed
    MalformedData,
    /// There is not factory for the specified type_id
    MissingFactory,
}

/// The data type of the number used to store the data type
type ObjectType = u8;
/// The number of bytes used to represent the data type
const OBJECT_TYPE_NUM_BYTES: usize = std::mem::size_of::<ObjectType>();

/// Converts a slice to a fixed size array unsafely
pub unsafe fn slice_to_array<T, const S: usize>(slice: &[T]) -> [T; S]
where
    [T; S]: for<'a> TryFrom<&'a [T]>,
{
    unsafe { slice.try_into().unwrap_unchecked() }
}

/// The number type that is used to determine the length of the text data type
type KeyLenType = u16;
/// The number of bytes used to store the length of the text data type
const KEY_LEN_TYPE_NUM_BYTES: usize = std::mem::size_of::<KeyLenType>();

/// What items are stored under in the database
// TODO: I don't care about the capacity of the string so maybe change to a len and u8 slice instead
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Key(String);

impl Key {
    /// Create a new key from a type that can be converted into a String
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

/// Extracts the key from a byte slice and returns what is remaining in the slice
// TODO: Should this be an associated function
pub fn extract_key(slice: &[u8]) -> Result<(Key, &[u8]), ObjectError> {
    trace!("Deserializing key");

    // first check if the size is large enough
    if slice.len() < KEY_LEN_TYPE_NUM_BYTES {
        return Err(ObjectError::MissingData);
    }

    // extracting text length
    let key_len =
        KeyLenType::from_be_bytes(unsafe { slice_to_array(&slice[..KEY_LEN_TYPE_NUM_BYTES]) })
            as usize;
    trace!("Key len: {key_len}");

    // making sure there is enough bytes left
    let slice = &slice[KEY_LEN_TYPE_NUM_BYTES..];

    if slice.len() < key_len {
        return Err(ObjectError::MissingData);
    }

    // try and convert byte slice to string
    let key = str::from_utf8(&slice[..key_len])
        .map_err(|_| ObjectError::MalformedData)?
        .to_owned();

    trace!("Extracted key: {key}");

    Ok((Key::new(key), &slice[key_len..]))
}

/// Anything that implements object is valid to store and retrieve from the database
pub trait Object: std::fmt::Debug {}

/// The raw bytes that an object can be built from
pub struct RawObjectData {
    raw_data: Vec<u8>,
}

impl RawObjectData {
    /// Creates a new RawObjectData from a byte slice
    /// No error checking is performed
    fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            raw_data: data.into(),
        }
    }
}

impl AsRef<[u8]> for RawObjectData {
    fn as_ref(&self) -> &[u8] {
        &self.raw_data
    }
}

/// Data that can be used to create an object
pub struct ObjectData {
    type_id: ObjectType,
    data: RawObjectData,
}

impl ObjectData {
    /// Creates a new object data from bytes
    /// Note the only checking that occurs is if the the byte slice is long enough to have a type_id
    /// This byte data should include the type id
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ObjectError> {
        // checking if there are enough bytes for the type id
        if bytes.len() < OBJECT_TYPE_NUM_BYTES {
            return Err(ObjectError::MissingData);
        }

        let type_id =
            ObjectType::from_be_bytes(unsafe { slice_to_array(&bytes[..OBJECT_TYPE_NUM_BYTES]) });

        Ok(Self {
            type_id,
            data: RawObjectData::new(&bytes[OBJECT_TYPE_NUM_BYTES..]),
        })
    }

    /// Returns a byte slice of the data that can be used to create objects
    pub fn data(self) -> RawObjectData {
        self.data
    }

    /// Extract the type id from the data
    pub fn type_id(&self) -> ObjectType {
        self.type_id
    }
}

/// Responsible for creating an object
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(RawObjectData) -> Result<Box<dyn Object>, ObjectError>,
{
    creator: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(RawObjectData) -> Result<Box<dyn Object>, ObjectError>,
{
    /// Creates a new object factory
    pub fn new(creator: F) -> Self {
        Self { creator }
    }

    /// Creates an object
    pub fn create(&self, data: RawObjectData) -> Result<Box<dyn Object>, ObjectError> {
        (self.creator)(data)
    }
}

/// Just shortand for the ObjectFactory that the TypeRegistry uses
type TypeRegistryFactoryType =
    ObjectFactory<Box<dyn Fn(RawObjectData) -> Result<Box<dyn Object>, ObjectError> + Sync + Send>>;

/// Responsible for holding and managing mappings of type ids to methods to create the types
pub struct TypeRegistry {
    registry: HashMap<ObjectType, TypeRegistryFactoryType>,
}

impl TypeRegistry {
    /// Creates a new type registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Inserts a factory into the registry with an associated type id
    pub fn add_factory(&mut self, type_id: ObjectType, factory: TypeRegistryFactoryType) {
        self.registry.insert(type_id, factory);
    }

    /// Gets a ObjectFactory from the registry
    pub fn get_factory(&self, type_id: ObjectType) -> Option<&TypeRegistryFactoryType> {
        self.registry.get(&type_id)
    }
}

impl Default for TypeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// There is only one type registry that must be shared with everything
static REGISTRY: LazyLock<RwLock<TypeRegistry>> =
    LazyLock::new(|| RwLock::new(TypeRegistry::default()));

/// Adds a new ObjectFactory to the type registry
pub fn register_factory(type_id: ObjectType, factory: TypeRegistryFactoryType) {
    REGISTRY.write().unwrap().add_factory(type_id, factory);
}

/// Creates a new object and derives the type id from the input data
pub fn new_object(object_data: ObjectData) -> Result<Box<dyn Object>, ObjectError> {
    // getting the factory
    let registry = REGISTRY.read().unwrap();
    let Some(factory) = registry.get_factory(object_data.type_id) else {
        return Err(ObjectError::MissingFactory);
    };

    factory.create(object_data.data())
}
