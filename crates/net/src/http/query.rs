use std::fmt;
use wasm_bindgen::UnwrapThrowExt;

/// A sequence of URL query parameters, wrapping [`web_sys::UrlSearchParams`].
pub struct QueryParams {
    raw: web_sys::UrlSearchParams,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryParams {
    /// Create a new empty query parameters object.
    pub fn new() -> Self {
        // pretty sure this will never throw.
        Self {
            raw: web_sys::UrlSearchParams::new().unwrap_throw(),
        }
    }

    /// Append a parameter to the query string.
    pub fn append(&self, name: &str, value: &str) {
        self.raw.append(name, value)
    }
}

impl fmt::Display for QueryParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw.to_string())
    }
}
