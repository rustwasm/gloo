//! All the errors.

use gloo_utils::errors::JsError;
use wasm_bindgen::{JsCast, JsValue};

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
        Ok(error) => StorageError::JsError(JsError::from(error)),
        Err(_) => unreachable!("JsValue passed is not an Error type - this is a bug"),
    }
}
