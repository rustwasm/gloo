#![allow(clippy::unused_unit)]
//! This crate provides wrappers for the
//! [Web Storage API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API)
//!
//! The data is stored in JSON form. We use [`serde`](https://serde.rs) for
//! serialization and deserialization.

#![deny(missing_docs, missing_debug_implementations)]

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::errors::js_to_error;
use errors::StorageError;
use gloo_utils::window;
use js_sys::Reflect;
use serde_json::{Map, Value};

#[macro_use]
mod macros;
pub mod errors;
pub mod indexed_db;
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

/// Have we been granted permission to store data indefinitely?
pub async fn is_persisted() -> bool {
    JsFuture::from(storage_manager().persisted().unwrap_throw())
        .await
        .unwrap_throw()
        .is_truthy()
}

/// Request that stored data be persisted and not reclaimed unless the user specifically clears
/// their storage.
///
/// Returns `true` if the request was granted, or `false` if not.
pub async fn persist() -> bool {
    JsFuture::from(storage_manager().persist().unwrap_throw())
        .await
        .unwrap_throw()
        .is_truthy()
}

/// How much quota do we have, and how much have we used?
pub async fn estimate() -> Quota {
    let raw = JsFuture::from(storage_manager().estimate().unwrap_throw())
        .await
        .unwrap_throw();
    // The casts here are lossy, but the values are approximate anyway.
    let total = Reflect::get(&raw, &JsValue::from_str(wasm_bindgen::intern("quota")))
        .unwrap_throw()
        .as_f64()
        .unwrap_throw() as u64;
    let used = Reflect::get(&raw, &JsValue::from_str(wasm_bindgen::intern("usage")))
        .unwrap_throw()
        .as_f64()
        .unwrap_throw() as u64;
    Quota { total, used }
}

/// Approximate amount of storage available, and used.
///
/// See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/estimate) for more
/// details.
#[derive(Debug, Copy, Clone)]
pub struct Quota {
    /// The total space available to this origin.
    pub total: u64,
    /// The space used by this origin.
    pub used: u64,
}

/// Get the user agent's storage manager instance
fn storage_manager() -> web_sys::StorageManager {
    window().navigator().storage()
}
