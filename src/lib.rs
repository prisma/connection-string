//! Connection string parsing in Rust
//!
//! # Examples
//!
//! JDBC
//! ```
//! use connection_string::JdbcString;
//!
//! let conn: JdbcString = r#"jdbc:sqlserver://server\instance:80;key=value;foo=bar"#.parse().unwrap();
//! assert_eq!(conn.sub_protocol(), "jdbc:sqlserver");
//! ```
//!
//! Ado.net
//! ```
//! use connection_string::AdoNetString;
//!
//! let input = "Persist Security Info=False;Integrated Security=true;\nInitial Catalog=AdventureWorks;Server=MSSQL1";
//! let _: AdoNetString = input.parse().unwrap();
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
