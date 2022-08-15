use wasm_bindgen::{JsValue, UnwrapThrowExt};

/// Creates a new `JsValue` from the JSON serialization of the object `t`
/// provided.
///
/// This function will serialize the provided value `t` to a JSON string,
/// send the JSON string to JS, parse it into a JS object, and then return
/// a handle to the JS object. This is unlikely to be super speedy so it's
/// not recommended for large payloads, but it's a nice to have in some
/// situations!
///
/// Usage of this API requires activating the `serde-serialize` feature of
/// the `gloo-utils` crate.
///
/// # Errors
///
/// Returns any error encountered when serializing `T` into JSON.
///
/// # Panics
///
/// Panics if `serde_json` generated json that couldn't be parsed by `js_sys`.
/// Uses `unwrap_throw` from `wasm_bindgen::UnwrapThrowExt`.
#[cfg(feature = "serde-serialize")]
pub fn from_serde<T>(t: &T) -> serde_json::Result<JsValue>
where
    T: serde::ser::Serialize + ?Sized,
{
    let s = serde_json::to_string(t)?;
    Ok(js_sys::JSON::parse(&s).unwrap_throw())
}

/// Invokes `JSON.stringify` on this value and then parses the resulting
/// JSON into an arbitrary Rust value.
///
/// This function will first call `JSON.stringify` on the `JsValue` itself.
/// The resulting string is then passed into Rust which then parses it as
/// JSON into the resulting value.
///
/// Usage of this API requires activating the `serde-serialize` feature of
/// the `gloo-utils` crate.
///
/// # Errors
///
/// Returns any error encountered when parsing the JSON into a `T`.
///
/// # Panics
///
/// Panics if `js_sys` couldn't stringify the JsValue. Uses `unwrap_throw`
/// from `wasm_bindgen::UnwrapThrowExt`.
#[cfg(feature = "serde-serialize")]
pub fn into_serde<T>(src: &JsValue) -> serde_json::Result<T>
where
    T: for<'a> serde::de::Deserialize<'a>,
{
    let s = if src.is_undefined() {
        String::new()
    } else {
        js_sys::JSON::stringify(src)
            .map(String::from)
            .unwrap_throw()
    };
    serde_json::from_str(&s)
}
