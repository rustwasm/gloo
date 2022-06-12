use gloo_utils::iter::UncheckedIter;
use js_sys::{Array, Map};
use std::fmt;
use wasm_bindgen::{JsCast, UnwrapThrowExt};

// I experimented with using `js_sys::Object` for the headers, since this object is marked
// experimental in MDN. However it's in the fetch spec, and it's necessary for appending headers.
/// A wrapper around `web_sys::Headers`.
pub struct Headers {
    raw: web_sys::Headers,
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

impl Headers {
    /// Create a new empty headers object.
    pub fn new() -> Self {
        // pretty sure this will never throw.
        Self {
            raw: web_sys::Headers::new().unwrap_throw(),
        }
    }

    /// Build [Headers] from [web_sys::Headers].
    pub fn from_raw(raw: web_sys::Headers) -> Self {
        Self { raw }
    }

    /// Covert [Headers] to [web_sys::Headers].
    pub fn into_raw(self) -> web_sys::Headers {
        self.raw
    }

    /// This method appends a new value onto an existing header, or adds the header if it does not
    /// already exist.
    pub fn append(&self, name: &str, value: &str) {
        // XXX Can this throw? WEBIDL says yes, my experiments with forbidden headers and MDN say
        // no.
        self.raw.append(name, value).unwrap_throw()
    }

    /// Deletes a header if it is present.
    pub fn delete(&self, name: &str) {
        self.raw.delete(name).unwrap_throw()
    }

    /// Gets a header if it is present.
    pub fn get(&self, name: &str) -> Option<String> {
        self.raw.get(name).unwrap_throw()
    }

    /// Whether a header with the given name exists.
    pub fn has(&self, name: &str) -> bool {
        self.raw.has(name).unwrap_throw()
    }

    /// Overwrites a header with the given name.
    pub fn set(&self, name: &str, value: &str) {
        self.raw.set(name, value).unwrap_throw()
    }

    /// Iterate over (header name, header value) pairs.
    pub fn entries(&self) -> impl Iterator<Item = (String, String)> {
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

    /// Iterate over the names of the headers.
    pub fn keys(&self) -> impl Iterator<Item = String> {
        let fake_map: &Map = self.raw.unchecked_ref();
        UncheckedIter::from(fake_map.keys()).map(|key| key.as_string().unwrap_throw())
    }

    /// Iterate over the values of the headers.
    pub fn values(&self) -> impl Iterator<Item = String> {
        let fake_map: &Map = self.raw.unchecked_ref();
        UncheckedIter::from(fake_map.values()).map(|v| v.as_string().unwrap_throw())
    }
}

impl fmt::Debug for Headers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut dbg = f.debug_struct("Headers");
        for (key, value) in self.entries() {
            dbg.field(&key, &value);
        }
        dbg.finish()
    }
}
