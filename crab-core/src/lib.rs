use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

pub mod int;
pub mod null;
pub mod text;

/// Errors that can occur when deseriarlizing
pub enum DeserializeError {
    /// The type specified is invalid
    InvalidType,
    /// The data provided for the type is invalid
    MalformedData,
}

/// Implementations of this trait must implement a way to convert some type to another type
pub trait Deserialize<T> {
    /// Converts from one type to another and returns that type along with what remains from the previous type
    fn deserialize(source: T) -> Result<Box<Self>, DeserializeError>;
}

/// Errors that can occur when seriarlizing
pub enum SerializeError {}

/// Implementations of this tait must implement a way to serialize the type
pub trait Serialize<T> {
    /// Converts from a concrete type to another type that can be stored or sent
    fn serialize(self) -> Result<T, SerializeError>;
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

impl<T> From<T> for Key
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Anything that implements object is valid to store and retrieve from the database
pub trait Object: std::fmt::Debug {}

/// Responsible for creating an object
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(&[u8]) -> Box<dyn Object>,
{
    creator: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(&[u8]) -> Box<dyn Object>,
{
    /// Creates a new object factory
    pub fn new(creator: F) -> Self {
        Self { creator }
    }

    /// Creates an object
    pub fn create(&self, data: &[u8]) -> Box<dyn Object> {
        (self.creator)(data)
    }
}

/// Just shortand for the ObjectFactory that the TypeRegistry uses
type TypeRegistryFactoryType = ObjectFactory<Box<dyn Fn(&[u8]) -> Box<dyn Object> + Sync + Send>>;

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

/// Create a new object using the appropriate ObjectFactory in the TypeRegistry
/// If there is not ObjectFactory for the type id None is returned
pub fn new_object(type_id: ObjectType, data: &[u8]) -> Option<Box<dyn Object>> {
    Some(REGISTRY.read().unwrap().get_factory(type_id)?.create(data))
}
