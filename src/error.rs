#[derive(Debug)]
pub struct Error {}

/// Create a new Error.
impl Error {
    pub fn new(_s: &str) -> Self {
        Self {}
    }
}
