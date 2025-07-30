/// This represents a null object in the database
/// so just no value
#[derive(Debug)]
pub struct Null;

impl Null {
    pub fn validate(_: &[u8]) -> bool {
        true
    }
}
