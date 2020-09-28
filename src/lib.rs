//! Connection string parsing in Rust
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, future_incompatible, unreachable_pub)]

mod ado;
mod error;
mod jdbc;

#[macro_use]
mod utils;

pub use ado::AdoNetString;
pub use jdbc::JdbcString;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
