use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug)]
/// A version of `JdbcString` to be used from web-assembly.
pub struct AdoNetString {
    inner: crate::ado::AdoNetString,
}

#[wasm_bindgen]
impl AdoNetString {
    #[wasm_bindgen(constructor)]
    /// A constructor to create a new `AdoNet`, used from JavaScript with
    /// `new AdoNet("server=tcp:localhost,1433")`.
    pub fn new(s: &str) -> Result<AdoNetString, JsValue> {
        let inner = s
            .parse()
            .map_err(|err| JsValue::from_str(&format!("{}", err)))?;

        Ok(Self { inner })
    }

    /// Get a parameter from the connection's key-value pairs
    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).map(|s| s.to_string())
    }

    /// Set a parameter value to the connection's key-value pairs. If replacing
    /// a pre-existing value, returns the old value.
    pub fn set(&mut self, key: &str, value: &str) -> Option<String> {
        self.inner.insert(key.into(), value.into())
    }

    /// Get a string representation of the `AdoNetString`.
    pub fn to_string(&self) -> String {
        format!("{}", self.inner)
    }
}
