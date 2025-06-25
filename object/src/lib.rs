use std::collections::HashMap;

/// The value under which an object is stored in the database
#[derive(Debug)]
pub struct Key(String);

/// Used to represent the type of the object
pub type TypeId = u8;

/// Defines and object as well as what methods can be performed on it
pub trait Object: Send + Sync + std::fmt::Debug {
    /// Returns the TypeId of the object
    fn type_id(&self) -> TypeId;

    /// Turn the object into raw bytes
    fn serialize(self) -> Vec<u8>;
}

/// Used to create Box<dyn Objects>
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> Box<dyn Object>,
{
    factory_method: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> Box<dyn Object>,
{
    /// Creates a new ObjectFactory
    pub fn new(factory_method: F) -> Self {
        Self { factory_method }
    }

    /// Creates a Box<dyn Object> from some bytes
    pub fn create_object(&self, bytes: Vec<u8>) -> Box<dyn Object> {
        (self.factory_method)(bytes)
    }
}

/// The type of the factory used in the Registry
type RegistryObjectFactory = ObjectFactory<Box<dyn Fn(Vec<u8>) -> Box<dyn Object>>>;

/// Contains a mapping of TypeId's to ObjectFactories and is used to ceate Box<dyn Object>'s
#[derive(Default)]
struct Registry {
    factories: HashMap<TypeId, RegistryObjectFactory>,
}

impl Registry {
    /// Adds an ObjectFactory to the Registry
    pub fn register_factory(&mut self, type_id: TypeId, factory: RegistryObjectFactory) {
        self.factories.insert(type_id, factory);
    }
}
