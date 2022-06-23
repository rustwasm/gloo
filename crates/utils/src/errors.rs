use std::convert::TryFrom;
use std::fmt;
use wasm_bindgen::{JsCast, JsValue};

/// Wrapper type around [`js_sys::Error`]
///
/// [`Display`][fmt::Display] impl returns the result `error.toString()` from JavaScript
pub struct JsError {
    /// `name` from [`js_sys::Error`]
    pub name: String,
    /// `message` from [`js_sys::Error`]
    pub message: String,
    js_to_string: String,
}

impl From<js_sys::Error> for JsError {
    fn from(error: js_sys::Error) -> Self {
        JsError {
            name: String::from(error.name()),
            message: String::from(error.message()),
            js_to_string: String::from(error.to_string()),
        }
    }
}

/// The [`JsValue`] is not a JavaScript's `Error`.
pub struct NotJsError {
    pub js_value: JsValue,
    js_to_string: String,
}

impl fmt::Debug for NotJsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NotJsError")
            .field("js_value", &self.js_value)
            .finish()
    }
}

impl fmt::Display for NotJsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.js_to_string)
    }
}

impl std::error::Error for NotJsError {}

impl TryFrom<JsValue> for JsError {
    type Error = NotJsError;

    fn try_from(value: JsValue) -> Result<Self, Self::Error> {
        match value.dyn_into::<js_sys::Error>() {
            Ok(error) => Ok(JsError::from(error)),
            Err(js_value) => {
                let js_to_string = String::from(js_sys::JsString::from(js_value.clone()));
                Err(NotJsError {
                    js_value,
                    js_to_string,
                })
            }
        }
    }
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.js_to_string)
    }
}

impl fmt::Debug for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsError")
            .field("name", &self.name)
            .field("message", &self.message)
            .finish()
    }
}

impl std::error::Error for JsError {}
