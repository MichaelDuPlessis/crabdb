/// What items are stored under in the database
// TODO: I don't care about the capacity of the string so maybe change to a len and u8 slice instead
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Key(String);

/// The Int data type. It is internally reprsented as an isize.
#[derive(Debug)]
pub struct Int(isize);

impl Int {
    /// Creates a new Int from an isize
    pub fn new(num: impl Into<isize>) -> Self {
        Self(num.into())
    }
}

/// The Text data type. It is internally reprsented as an String.
#[derive(Debug)]
pub struct Text(String);

impl Text {
    /// Creates a new Text from a String
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
}

/// The available data types for the database
#[derive(Debug)]
pub enum Object {
    Int(Int),
    Text(Text),
    // Struct,
    // List,
}
