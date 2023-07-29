#![cfg(feature = "serde")]

use wasm_bindgen::{JsValue, UnwrapThrowExt};
mod private {
    pub trait Sealed {}
    impl Sealed for wasm_bindgen::JsValue {}
}

/// Extension trait to provide conversion between [`JsValue`](wasm_bindgen::JsValue) and [`serde`].
///
/// Usage of this API requires activating the `serde` feature of the `gloo-utils` crate.
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub trait JsValueSerdeExt: private::Sealed {
    /// Creates a new `JsValue` from the JSON serialization of the object `t`
    /// provided.
    ///
    /// This function will serialize the provided value `t` to a JSON string,
    /// send the JSON string to JS, parse it into a JS object, and then return
    /// a handle to the JS object. This is unlikely to be super speedy so it's
    /// not recommended for large payloads, but it's a nice to have in some
    /// situations!
    ///
    /// Usage of this API requires activating the `serde` feature of
    /// the `gloo-utils` crate.
    /// # Example
    ///
    /// ```rust
    /// use wasm_bindgen::JsValue;
    /// use gloo_utils::format::JsValueSerdeExt;
    ///
    /// # fn no_run() {
    /// let array = vec![1,2,3];
    /// let obj = JsValue::from_serde(&array);
    /// # }
    /// ```
    /// # Errors
    ///
    /// Returns any error encountered when serializing `T` into JSON.
    ///
    /// # Panics
    ///
    /// Panics if [`serde_json`](serde_json::to_string) generated JSON that couldn't be parsed by [`js_sys`].
    /// Uses [`unwrap_throw`](UnwrapThrowExt::unwrap_throw) from [`wasm_bindgen::UnwrapThrowExt`].
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    fn from_serde<T>(t: &T) -> serde_json::Result<JsValue>
    where
        T: serde::ser::Serialize + ?Sized;

    /// Invokes `JSON.stringify` on this value and then parses the resulting
    /// JSON into an arbitrary Rust value.
    ///
    /// This function will first call `JSON.stringify` on the `JsValue` itself.
    /// The resulting string is then passed into Rust which then parses it as
    /// JSON into the resulting value. If given `undefined`, object will be silently changed to
    /// null to avoid panic.
    ///
    /// Usage of this API requires activating the `serde` feature of
    /// the `gloo-utils` crate.
    ///
    /// # Example
    ///
    /// ```rust
    /// use wasm_bindgen::JsValue;
    /// use gloo_utils::format::JsValueSerdeExt;
    ///
    /// # fn no_run() {
    /// assert_eq!(JsValue::from("bar").into_serde::<String>().unwrap(), "bar");
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns any error encountered when parsing the JSON into a `T`.
    ///
    /// # Panics
    ///
    /// Panics if [`js_sys`] couldn't stringify the JsValue. Uses [`unwrap_throw`](UnwrapThrowExt::unwrap_throw)
    /// from [`wasm_bindgen::UnwrapThrowExt`].
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    #[allow(clippy::wrong_self_convention)]
    fn into_serde<T>(&self) -> serde_json::Result<T>
    where
        T: for<'a> serde::de::Deserialize<'a>;
}

impl JsValueSerdeExt for JsValue {
    fn from_serde<T>(t: &T) -> serde_json::Result<JsValue>
    where
        T: serde::ser::Serialize + ?Sized,
    {
        let s = serde_json::to_string(t)?;
        Ok(js_sys::JSON::parse(&s).unwrap_throw())
    }

    fn into_serde<T>(&self) -> serde_json::Result<T>
    where
        T: for<'a> serde::de::Deserialize<'a>,
    {
        // Turns out `JSON.stringify(undefined) === undefined`, so if
        // we're passed `undefined` reinterpret it as `null` for JSON
        // purposes.
        let s = if self.is_undefined() {
            String::from("null")
        } else {
            js_sys::JSON::stringify(self)
                .map(String::from)
                .unwrap_throw()
        };
        serde_json::from_str(&s)
    }
}
