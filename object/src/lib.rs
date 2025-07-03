use core::error;
use std::{
    collections::HashMap,
    fmt,
    sync::{LazyLock, RwLock},
};

pub mod int;
pub mod null;

/// The value under which an object is stored in the database
#[derive(Debug)]
pub struct Key(String);

/// Used to represent the type of the object
pub type TypeId = u8;

/// The type of errors that can occur when constructing an object
#[derive(Debug)]
pub struct ObjectError;

impl fmt::Display for ObjectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the data provided for the Object is invalid")
    }
}

impl error::Error for ObjectError {}

/// Defines and object as well as what methods can be performed on it
pub trait Object: std::fmt::Debug {
    /// Returns the TypeId of the object
    fn type_id(&self) -> TypeId;

    /// Turn the object into raw bytes
    fn serialize(self) -> Vec<u8>;

    /// Turn raw bytes into an object
    fn deserialize(bytes: impl AsRef<[u8]>) -> Result<DbObject, ObjectError>
    where
        Self: Sized;
}

/// The type of the object used in the database
pub type DbObject = Box<dyn Object + Send + Sync>;

/// Used to create Box<dyn Objects>
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(&[u8]) -> Result<DbObject, ObjectError>,
{
    factory_method: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(&[u8]) -> Result<DbObject, ObjectError>,
{
    /// Creates a new ObjectFactory
    pub fn new(factory_method: F) -> Self {
        Self { factory_method }
    }

    /// Creates a Box<dyn Object> from some bytes
    pub fn create_object(&self, bytes: impl AsRef<[u8]>) -> Result<DbObject, ObjectError> {
        (self.factory_method)(bytes.as_ref())
    }
}

/// The kinds of errors that can occur with the registry
#[derive(Debug)]
pub enum RegistryError {
    /// The factory for the TypeId specified does not exist
    NoFactory,
    /// The TypeId being registered has already been registered
    AlreadyRegistered,
    /// The Object failed to create
    ObjectError,
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                RegistryError::NoFactory => "no factory found for specified TypeId",
                RegistryError::AlreadyRegistered =>
                    "the TypeId specified has already been registered",
                RegistryError::ObjectError => "the Object failed to create",
            }
        )
    }
}

impl std::error::Error for RegistryError {}

impl From<ObjectError> for RegistryError {
    fn from(_: ObjectError) -> Self {
        Self::ObjectError
    }
}

/// The type of the factory used in the Registry
type RegistryObjectFactory =
    ObjectFactory<Box<dyn Fn(&[u8]) -> Result<DbObject, ObjectError> + Send + Sync>>;

/// Contains a mapping of TypeId's to ObjectFactories and is used to ceate Box<dyn Object>'s
#[derive(Default)]
struct Registry {
    factories: HashMap<TypeId, RegistryObjectFactory>,
}

impl Registry {
    /// Adds an ObjectFactory to the Registry
    pub fn register_factory(
        &mut self,
        type_id: TypeId,
        factory: RegistryObjectFactory,
    ) -> Result<(), RegistryError> {
        if self.factories.contains_key(&type_id) {
            Err(RegistryError::AlreadyRegistered)
        } else {
            self.factories.insert(type_id, factory);
            Ok(())
        }
    }

    /// Creates an object using the Registry and the associated ObjectFactory if one exists
    pub fn create_object(
        &self,
        type_id: TypeId,
        bytes: impl AsRef<[u8]>,
    ) -> Result<DbObject, RegistryError> {
        if let Some(factory) = self.factories.get(&type_id) {
            Ok(factory.create_object(bytes)?)
        } else {
            Err(RegistryError::NoFactory)
        }
    }
}

/// There should only ever be one registry struct
static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(Default::default);

/// All methods for a registry
pub mod type_registry {
    use crate::{DbObject, REGISTRY, RegistryError, RegistryObjectFactory, TypeId};

    /// Register a new type
    pub fn register_factory(
        type_id: TypeId,
        factory: RegistryObjectFactory,
    ) -> Result<(), RegistryError> {
        // getting the registry
        let mut registry = REGISTRY.write().unwrap();
        registry.register_factory(type_id, factory)
    }

    /// Create a new object from raw bytes the TypeId is extracted from the bytes
    pub fn create_object(
        type_id: TypeId,
        bytes: impl AsRef<[u8]>,
    ) -> Result<DbObject, RegistryError> {
        let registry = REGISTRY.read().unwrap();
        registry.create_object(type_id, bytes)
    }
}
