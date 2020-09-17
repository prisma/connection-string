#[derive(Debug)]
pub struct Error {
    msg: String,
}

/// Create a new Error.
impl Error {
    pub fn new(msg: &str) -> Self {
        Self {msg: msg.to_owned() }
    }
}
