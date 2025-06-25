/// The value under which an object is stored in the database
pub struct Key(String);

/// Defines and object as well as what methods can be performed on it
pub trait Object: Send + Sync {}
