//! All the errors.

use std::fmt;

use wasm_bindgen::{JsCast, JsValue};

/// Error returned from JavaScript
pub struct JsError {
    /// `name` field of JavaScript's error
    pub name: String,
    /// `message` field of JavaScript's error
    pub message: String,
    js_to_string: String,
}

impl fmt::Debug for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsError")
            .field("name", &self.name)
            .field("message", &self.message)
            .finish()
    }
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.js_to_string)
    }
}

/// Error returned by this crate
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Error from `serde`
    #[error("{0}")]
    SerdeError(#[from] serde_json::Error),
    /// Error if the requested key is not found
    #[error("key {0} not found")]
    KeyNotFound(String),
    /// Error returned from JavaScript
    #[error("{0}")]
    JsError(JsError),
}

pub(crate) fn js_to_error(js_value: JsValue) -> StorageError {
    match js_value.dyn_into::<js_sys::Error>() {
        Ok(error) => StorageError::JsError(JsError {
            name: String::from(error.name()),
            message: String::from(error.message()),
            js_to_string: String::from(error.to_string()),
        }),
        Err(_) => unreachable!("JsValue passed is not an Error type - this is a bug"),
    }
}
