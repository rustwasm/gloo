//! This crate provides wrappers for the
//! [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
//!
//! The data is stored in JSON form. We use [`serde`](https://serde.rs) for
//! serialization and deserialization.

#![deny(missing_docs, missing_debug_implementations)]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::errors::js_to_error;
use errors::StorageError;
use serde_json::{Map, Value};

pub mod errors;
mod local_storage;
mod session_storage;
pub use local_storage::LocalStorage;
pub use session_storage::SessionStorage;

/// `gloo-storage`'s `Result`
pub type Result<T> = std::result::Result<T, StorageError>;

/// Trait which provides implementations for managing storage in the browser.
pub trait Storage {
    /// Get the raw [`web_sys::Storage`] instance
    fn raw() -> web_sys::Storage;

    /// Get the value for the specified key
    fn get<T>(key: impl AsRef<str>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let key = key.as_ref();
        let item = Self::raw()
            .get_item(key)
            .expect_throw("unreachable: get_item does not throw an exception")
            .ok_or_else(|| StorageError::KeyNotFound(key.to_string()))?;
        let item = serde_json::from_str(&item)?;
        Ok(item)
    }

    /// Get all the stored keys and their values
    fn get_all<T>() -> Result<T>
    where
        T: for<'a> Deserialize<'a>,
    {
        let local_storage = Self::raw();
        let length = Self::length();
        let mut map = Map::with_capacity(length as usize);
        for index in 0..length {
            let key = local_storage
                .key(index)
                .map_err(js_to_error)?
                .unwrap_throw();
            let value: Value = Self::get(&key)?;
            map.insert(key, value);
        }
        Ok(serde_json::from_value(Value::Object(map))?)
    }

    /// Insert a value for the specified key
    fn set<T>(key: impl AsRef<str>, value: T) -> Result<()>
    where
        T: Serialize,
    {
        let key = key.as_ref();
        let value = serde_json::to_string(&value)?;
        Self::raw()
            .set_item(key, &value)
            .map_err(errors::js_to_error)?;
        Ok(())
    }

    /// Remove a key and it's stored value
    fn delete(key: impl AsRef<str>) {
        let key = key.as_ref();
        Self::raw()
            .remove_item(key)
            .expect_throw("unreachable: remove_item does not throw an exception");
    }

    /// Remove all the stored data
    fn clear() {
        Self::raw()
            .clear()
            .expect_throw("unreachable: clear does not throw an exception");
    }

    /// Get the number of items stored
    fn length() -> u32 {
        Self::raw()
            .length()
            .expect_throw("unreachable: length does not throw an exception")
    }
}
