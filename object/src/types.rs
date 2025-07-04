pub mod int;
pub mod null;
pub mod text;

pub mod type_ids {
    /// Used to represent the type of the object
    pub type TypeId = u8;
    /// The amount of bytes TypeId requires
    pub const TYPE_ID_NUM_BYTES: usize = std::mem::size_of::<TypeId>();

    /// The TypeId for Null
    pub const NULL_TYPE_ID: TypeId = 0;
    /// The TypeId for Int
    pub const INT_TYPE_ID: TypeId = 1;
    /// The TypeId for Text
    pub const TEXT_TYPE_ID: TypeId = 2;
}
