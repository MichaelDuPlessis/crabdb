use crate::object::Object;

/// This struct represents nothing as in the absence of a value
#[derive(Debug, Clone)]
pub struct Null;

impl Object for Null {
    fn boxed_clone(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn into_raw(&self) -> Vec<u8> {
        Vec::with_capacity(0)
    }

    fn type_name(&self) -> &'static str {
        "null"
    }
}
