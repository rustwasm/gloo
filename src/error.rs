use wasm_bindgen::{JsValue, JsCast};
use js_sys::TypeError;
use thiserror::Error;

/// All the errors returned by this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// Generic error returned by JavaScript.
    #[error("{0:?}")]
    JsError(JsValue),
    /// TypeError returned by JavaScript.
    #[error("{0:?}")]
    TypeError(TypeError),
    /// Error returned by `serde` during deserialization.
    #[error("{0}")]
    SerdeError(
        #[source]
        #[from]
        serde_json::Error,
    ),
    /// Unknown error.
    #[error("{0}")]
    Other(anyhow::Error),
}

pub(crate) fn js_to_error(js_value: JsValue) -> Error {
    match js_value.dyn_into::<js_sys::TypeError>() {
        Ok(type_error) => Error::TypeError(type_error),
        Err(val) => Error::JsError(val),
    }
}
