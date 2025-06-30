use std::{
    collections::HashMap,
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
pub enum ObjectError {
    /// The data provided to create the object is invalid
    BadData,
}

/// Defines and object as well as what methods can be performed on it
pub trait Object: std::fmt::Debug {
    /// Returns the TypeId of the object
    fn type_id(&self) -> TypeId;

    /// Turn the object into raw bytes
    fn serialize(self) -> Vec<u8>;

    /// Turn raw bytes into an object
    fn deserialize(bytes: Vec<u8>) -> Result<DbObject, ObjectError>
    where
        Self: Sized;
}

/// The type of the object used in the database
pub type DbObject = Box<dyn Object + Send + Sync>;

/// Used to create Box<dyn Objects>
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> DbObject,
{
    factory_method: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> DbObject,
{
    /// Creates a new ObjectFactory
    pub fn new(factory_method: F) -> Self {
        Self { factory_method }
    }

    /// Creates a Box<dyn Object> from some bytes
    pub fn create_object(&self, bytes: Vec<u8>) -> DbObject {
        (self.factory_method)(bytes)
    }
}

/// The kinds of errors that can occur with the registry
pub enum RegistryError {
    /// The factory for the TypeId specified does not exist
    NoFactory,
    /// The TypeId being registered has already been registered
    AlreadyRegistered,
}

/// The type of the factory used in the Registry
type RegistryObjectFactory = ObjectFactory<Box<dyn Fn(Vec<u8>) -> DbObject + Send + Sync>>;

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
        bytes: Vec<u8>,
    ) -> Result<DbObject, RegistryError> {
        if let Some(factory) = self.factories.get(&type_id) {
            Ok(factory.create_object(bytes))
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
    pub fn register_type(
        type_id: TypeId,
        factory: RegistryObjectFactory,
    ) -> Result<(), RegistryError> {
        // getting the registry
        let mut registry = REGISTRY.write().unwrap();
        registry.register_factory(type_id, factory)
    }

    /// Create a new object from raw bytes the TypeId is extracted from the bytes
    pub fn create_object(type_id: TypeId, bytes: Vec<u8>) -> Result<DbObject, RegistryError> {
        let registry = REGISTRY.read().unwrap();
        registry.create_object(type_id, bytes)
    }
}
