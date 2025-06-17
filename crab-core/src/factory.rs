use crate::{
    ObjectError,
    object::{DbObject, Object, ObjectData, TypeId},
    types::null::Null,
};
use std::{
    collections::HashMap,
    sync::{LazyLock, RwLock},
};

/// Responsible for creating an object
#[derive(Debug)]
pub struct ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> Result<Box<dyn Object + Send + Sync>, ObjectError>,
{
    /// The name of the data type that the factory makes
    type_name: &'static str,
    /// The function that creates the Object
    creator: F,
}

impl<F> ObjectFactory<F>
where
    F: Fn(Vec<u8>) -> Result<Box<dyn Object + Send + Sync>, ObjectError>,
{
    /// Creates a new object factory
    pub fn new(type_name: &'static str, creator: F) -> Self {
        Self { type_name, creator }
    }

    /// Get the type name
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Creates an object
    pub fn create(&self, data: Vec<u8>) -> Result<Box<dyn Object + Send + Sync>, ObjectError> {
        (self.creator)(data)
    }
}

/// Just shortand for the ObjectFactory that the TypeRegistry uses
type TypeRegistryFactoryType = ObjectFactory<
    Box<dyn Fn(Vec<u8>) -> Result<Box<dyn Object + Send + Sync>, ObjectError> + Sync + Send>,
>;

/// Responsible for holding and managing mappings of type ids to methods to create the types
pub struct TypeRegistry {
    registry: HashMap<TypeId, TypeRegistryFactoryType>,
}

impl TypeRegistry {
    /// Creates a new type registry
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }

    /// Inserts a factory into the registry with an associated type id
    pub fn add_factory(&mut self, type_id: TypeId, factory: TypeRegistryFactoryType) {
        self.registry.insert(type_id, factory);
    }

    /// Gets a ObjectFactory from the registry
    pub fn get_factory(&self, type_id: TypeId) -> Option<&TypeRegistryFactoryType> {
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
/// It is assumed the the type with TypeId 0 is the null type
pub fn register_factory(type_id: TypeId, factory: TypeRegistryFactoryType) {
    REGISTRY.write().unwrap().add_factory(type_id, factory);
}

/// Creates a new object and derives the type id from the input data
pub fn new_db_object(object_data: ObjectData) -> Result<DbObject, ObjectError> {
    // getting the factory
    let registry = REGISTRY.read().unwrap();

    let type_id = object_data.type_id();
    let data = object_data.data();

    let Some(factory) = registry.get_factory(type_id) else {
        return Err(ObjectError::MissingFactory);
    };

    let object = factory.create(data)?;

    Ok(DbObject::new(type_id, object))
}

/// Creates a Null object
pub fn new_null_db_object() -> DbObject {
    DbObject::new(0, Box::new(Null))
}
