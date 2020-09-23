use std::fmt::{self, Display};

/// A connection string error.
#[derive(Debug)]
pub struct Error {
    msg: String,
}

/// Create a new Error.
impl Error {
    /// Create a new instance of `Error`.
    pub fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Self {
            msg: format!("{}", err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}
