use gloo_utils::iter::UncheckedIter;
use js_sys::{Array, Map};
use std::fmt;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

/// A sequence of URL query parameters, wrapping [`web_sys::UrlSearchParams`].
pub struct QueryParams {
    raw: web_sys::UrlSearchParams,
}

impl Default for QueryParams {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl QueryParams {
    /// Create a new empty query parameters object.
    pub fn new() -> Self {
        // pretty sure this will never throw.
        Self {
            raw: web_sys::UrlSearchParams::new().unwrap_throw(),
        }
    }

    /// Create [`QueryParams`] from [`web_sys::UrlSearchParams`] object.
    pub fn from_raw(raw: web_sys::UrlSearchParams) -> Self {
        Self { raw }
    }

    /// Append a parameter to the query string.
    pub fn append(&self, name: &str, value: &str) {
        self.raw.append(name, value)
    }

    /// Get the value of a parameter. If the parameter has multiple occurrences, the first value is
    /// returned.
    pub fn get(&self, name: &str) -> Option<String> {
        self.raw.get(name)
    }

    /// Get all associated values of a parameter.
    pub fn get_all(&self, name: &str) -> Vec<String> {
        self.raw
            .get_all(name)
            .iter()
            .map(|jsval| jsval.as_string().unwrap_throw())
            .collect()
    }

    /// Remove all occurrences of a parameter from the query string.
    pub fn delete(&self, name: &str) {
        self.raw.delete(name)
    }

    /// Iterate over (name, value) pairs of the query parameters.
    pub fn iter(&self) -> impl Iterator<Item = (String, String)> {
        // Here we cheat and cast to a map even though `self` isn't, because the method names match
        // and everything works. Is there a better way? Should there be a `MapLike` or
        // `MapIterator` type in `js_sys`?
        let fake_map: &Map = self.raw.unchecked_ref();
        UncheckedIter::from(fake_map.entries()).map(|entry| {
            let entry: Array = entry.unchecked_into();
            let key = entry.get(0);
            let value = entry.get(1);
            (
                key.as_string().unwrap_throw(),
                value.as_string().unwrap_throw(),
            )
        })
    }
}

/// The formatted query parameters ready to be used in a URL query string.
///
/// # Examples
///
/// The resulting string does not contain a leading `?` and is properly encoded:
///
/// ```
/// # fn no_run() {
/// use gloo_net::http::QueryParams;
///
/// let params = QueryParams::new();
/// params.append("a", "1");
/// params.append("b", "2");
/// assert_eq!(params.to_string(), "a=1&b=2".to_string());
///
/// params.append("key", "ab&c");
/// assert_eq!(params.to_string(), "a=1&b=2&key=ab%26c");
/// # }
/// ```
impl fmt::Display for QueryParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw.to_string())
    }
}

impl fmt::Debug for QueryParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
