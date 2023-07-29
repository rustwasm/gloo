use gloo_utils::errors::JsError;
use thiserror::Error as ThisError;

/// All the errors returned by this crate.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Error returned by JavaScript.
    #[error("{0}")]
    JsError(JsError),
    /// Error returned by `serde` during deserialization.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    #[error("{0}")]
    SerdeError(
        #[source]
        #[from]
        serde_json::Error,
    ),
    /// Error returned by this crate
    #[error("{0}")]
    GlooError(String),
}

#[cfg(any(feature = "http", feature = "websocket", feature = "eventsource"))]
pub(crate) use conversion::*;
#[cfg(any(feature = "http", feature = "websocket", feature = "eventsource"))]
mod conversion {
    use gloo_utils::errors::JsError;
    use std::convert::TryFrom;
    use wasm_bindgen::JsValue;

    #[cfg(feature = "http")]
    pub(crate) fn js_to_error(js_value: JsValue) -> super::Error {
        super::Error::JsError(js_to_js_error(js_value))
    }

    pub(crate) fn js_to_js_error(js_value: JsValue) -> JsError {
        match JsError::try_from(js_value) {
            Ok(error) => error,
            Err(_) => unreachable!("JsValue passed is not an Error type -- this is a bug"),
        }
    }
}
