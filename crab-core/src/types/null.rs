use crate::object::{Object, RawObjectData};

/// This struct represents nothing as in the absence of a value
#[derive(Debug, Clone)]
pub struct Null;

impl Object for Null {
    fn boxed_clone(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn into_raw_object_data(&self) -> RawObjectData {
        RawObjectData::new([])
    }
}
