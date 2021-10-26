use gloo_utils::errors::JsError;
use thiserror::Error as ThisError;
use wasm_bindgen::JsValue;

/// All the errors returned by this crate.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error returned by JavaScript.
    #[error("{0}")]
    JsError(JsError),
    /// Error returned by `serde` during deserialization.
    #[error("{0}")]
    SerdeError(
        #[source]
        #[from]
        serde_json::Error,
    ),
}

pub(crate) fn js_to_error(js_value: JsValue) -> Error {
    Error::JsError(js_to_js_error(js_value))
}

pub(crate) fn js_to_js_error(js_value: JsValue) -> JsError {
    match JsError::try_from(js_value) {
        Ok(error) => error,
        Err(_) => unreachable!("JsValue passed is not an Error type -- this is a bug"),
    }
}
