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

impl From<isize> for Int {
    fn from(value: isize) -> Self {
        Self::new(value)
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

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
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

impl Object {
    /// Creates a new Int type
    pub fn new_int<T>(num: T) -> Self
    where
        T: Into<Int>,
    {
        Self::Int(num.into())
    }

    /// Creates a new Text type
    pub fn new_text<T>(text: T) -> Self
    where
        T: Into<Text>,
    {
        Self::Text(text.into())
    }
}
