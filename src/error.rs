use std::fmt;
use thiserror::Error as ThisError;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug)]
pub struct JsError {
    pub name: String,
    pub message: String,
    js_to_string: String,
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.js_to_string)
    }
}

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
    /// Unknown error.
    #[error("{0}")]
    Other(anyhow::Error),
}

pub(crate) fn js_to_error(js_value: JsValue) -> Error {
    match js_value.dyn_into::<js_sys::Error>() {
        Ok(error) => Error::JsError(JsError {
            name: String::from(error.name()),
            message: String::from(error.message()),
            js_to_string: String::from(error.to_string()),
        }),
        Err(_) => unreachable!("JsValue passed is not an Error type -- this is a bug"),
    }
}
