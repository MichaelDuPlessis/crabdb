use crate::slice_to_array;

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
pub type TypeId = u8;
/// The number of bytes used to represent the data type
const TYPE_ID_NUM_BYTES: usize = std::mem::size_of::<TypeId>();

/// Anything that implements object is valid to store and retrieve from the database
pub trait Object: std::fmt::Debug {
    /// Creates a copy of a boxed object
    fn boxed_clone(&self) -> Box<dyn Object>;

    /// Convert the Object to the objects raw data
    fn into_raw_object_data(&self) -> RawObjectData;

    /// Return the type_name for the object
    fn type_name(&self) -> &'static str;
}

impl Clone for Box<dyn Object> {
    fn clone(&self) -> Self {
        self.boxed_clone()
    }
}

/// This is an object that is stored in the db
#[derive(Debug)]
pub struct DbObject {
    type_id: TypeId,
    object: Box<dyn Object>,
}

impl DbObject {
    /// Creates a new DbObject
    pub fn new(type_id: TypeId, object: Box<dyn Object>) -> Self {
        Self { type_id, object }
    }

    /// Convert the object data to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let raw_data = self.object.into_raw_object_data();
        let raw_data = raw_data.as_ref();
        let mut data = Vec::with_capacity(TYPE_ID_NUM_BYTES + raw_data.len());

        data.extend(self.type_id.to_be_bytes());
        data.extend(raw_data);

        data
    }
}

impl Clone for DbObject {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            object: self.object.boxed_clone(),
        }
    }
}

/// The raw bytes that an object can be built from
#[derive(Debug)]
pub struct RawObjectData {
    raw_data: Vec<u8>,
}

impl RawObjectData {
    /// Creates a new RawObjectData from a byte slice
    /// No error checking is performed
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
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
#[derive(Debug)]
pub struct ObjectData {
    type_id: TypeId,
    data: RawObjectData,
}

impl ObjectData {
    /// Creates a new object data from bytes
    /// Note the only checking that occurs is if the the byte slice is long enough to have a type_id
    /// This byte data should include the type id
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ObjectError> {
        // checking if there are enough bytes for the type id
        if bytes.len() < TYPE_ID_NUM_BYTES {
            return Err(ObjectError::MissingData);
        }

        let type_id = TypeId::from_be_bytes(unsafe { slice_to_array(&bytes[..TYPE_ID_NUM_BYTES]) });

        Ok(Self {
            type_id,
            data: RawObjectData::new(&bytes[TYPE_ID_NUM_BYTES..]),
        })
    }

    /// Returns a byte slice of the data that can be used to create objects
    pub fn data(self) -> RawObjectData {
        self.data
    }

    /// Extract the type id from the data
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Convert the object data to raw bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let raw_data = self.data.as_ref();
        let mut data = Vec::with_capacity(TYPE_ID_NUM_BYTES + raw_data.len());

        data.extend(self.type_id.to_be_bytes());
        data.extend(raw_data);

        data
    }
}

impl From<DbObject> for ObjectData {
    fn from(value: DbObject) -> Self {
        let object = value.object;

        Self {
            type_id: value.type_id,
            data: object.into_raw_object_data(),
        }
    }
}
