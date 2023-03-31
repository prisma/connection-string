use connection_string::JdbcString as BaseJdbcString;
use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
/// A version of `JdbcString` to be used from web-assembly.
pub struct JdbcString {
    inner: BaseJdbcString,
}

#[wasm_bindgen]
impl JdbcString {
    #[wasm_bindgen(constructor)]
    /// A constructor to create a new `JdbcInstance`, used from JavaScript with
    /// `new JdbcString("sqlserver://...")`.
    pub fn new(s: &str) -> Result<JdbcString, JsValue> {
        let inner = if s.starts_with("jdbc") {
            s.parse()
        } else {
            format!("jdbc:{}", s).parse()
        }
        .map_err(|err| JsValue::from_str(&format!("{}", err)))?;

        Ok(Self { inner })
    }

    /// Access the connection sub-protocol
    pub fn sub_protocol(&self) -> String {
        self.inner.sub_protocol().to_string()
    }

    /// Access the connection server name
    pub fn server_name(&self) -> Option<String> {
        self.inner.server_name().map(|s| s.to_string())
    }

    /// Access the connection's instance name
    pub fn instance_name(&self) -> Option<String> {
        self.inner.instance_name().map(|s| s.to_string())
    }

    /// Access the connection's port
    pub fn port(&self) -> Option<u16> {
        self.inner.port()
    }

    /// Get all keys from the connection's key-value pairs
    pub fn keys(&self) -> Array {
        self.inner
            .keys()
            .map(|k| JsValue::from(k))
            .collect::<Array>()
    }

    /// Get a parameter from the connection's key-value pairs
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.properties().get(key).map(|s| s.to_string())
    }

    /// Set a parameter value to the connection's key-value pairs. If replacing
    /// a pre-existing value, returns the old value.
    pub fn set(&mut self, key: &str, value: &str) -> Option<String> {
        self.inner.properties_mut().insert(key.into(), value.into())
    }

    /// Get a string representation of the `JdbcString`.
    pub fn to_string(&self) -> String {
        format!("{}", self.inner)
    }
}
