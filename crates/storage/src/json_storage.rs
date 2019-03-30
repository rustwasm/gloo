use crate::{Index, Storage, TableDefinition, TableManager, Version};
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

/// Common storage wrapper for both local and session storage.
#[derive(Debug)]
pub struct JsonStorage {
    database_json: Value,
    database_key: String,
    storage: web_sys::Storage,
}

impl JsonStorage {
    /// Creates a new instance with local storage.
    pub fn with_local_storage<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let storage = web_sys::window()
            .unwrap_throw()
            .local_storage()
            .unwrap_throw()
            .unwrap_throw();
        Self::instance(name.into(), storage)
    }

    /// Creates a new instance with session storage.
    pub fn with_session_storage<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let storage = web_sys::window()
            .unwrap_throw()
            .session_storage()
            .unwrap_throw()
            .unwrap_throw();
        Self::instance(name.into(), storage)
    }

    fn instance(database_key: String, storage: web_sys::Storage) -> Self {
        let opt_database_string = storage.get_item(&database_key).unwrap_throw();
        let database_json = if let Some(database_string) = opt_database_string {
            let database_json: Value = database_string.into();
            assert!(database_json.is_object());
            database_json
        } else {
            Map::new().into()
        };
        Self {
            database_json,
            database_key,
            storage,
        }
    }
}

impl Storage for JsonStorage {
    type TableManager = JsonStorageTableManager;
    type Version = JsonStorageVersion;

    fn add_version<F, I>(&mut self, version: I, cb: F)
    where
        F: FnOnce(Self::Version) -> Self::Version + 'static,
        I: Into<f64>,
    {
        let database_map = self.database_json.as_object_mut().unwrap();
        let version_f64 = version.into();
        let value_map = if database_map.is_empty() {
            Map::new()
        } else {
            let mut iter = database_map.iter();
            let (mut previous_version_key, _) = iter.next().unwrap();
            let mut previous_version_key_f64 = previous_version_key.parse::<f64>().unwrap();
            for (key, _) in database_map.iter() {
                let parsed_key = key.parse::<f64>().unwrap();
                if parsed_key < previous_version_key_f64 {
                    previous_version_key = key;
                    previous_version_key_f64 = parsed_key;
                }
            }
            if previous_version_key_f64 > version_f64 {
                panic!("A new version must have an id greater than all the other stored versions.")
            }
            database_map[previous_version_key]
                .as_object()
                .unwrap()
                .clone()
        };
        let jsv = cb(JsonStorageVersion { value_map });
        database_map.insert(version_f64.to_string(), jsv.value_map.into());
    }

    fn delete(&mut self) {
        self.storage.remove_item(&self.database_key).unwrap_throw();
    }

    fn name(&self) -> &str {
        &self.database_key
    }

    fn table_manager(&mut self, _: &str) -> Self::TableManager {
        unimplemented!();
    }

    fn transaction(&mut self) {
        unimplemented!();
    }
}

impl Drop for JsonStorage {
    fn drop(&mut self) {
        self.storage
            .set_item(&self.database_key, &self.database_json.to_string())
            .unwrap_throw();
    }
}

/// Storage table definition
#[derive(Debug)]
pub struct JsonStorageTableDefinition {
    pub(crate) table_map: Value,
}

impl TableDefinition for JsonStorageTableDefinition {
    /// The index has no effect as `Storage` does not support indexes.
    fn add_row_with_index(mut self, name: &str, _: Index) -> Self {
        self.table_map
            .as_object_mut()
            .unwrap()
            .insert(name.into(), Map::new().into());
        self
    }

    fn remove_old_row(mut self, name: &str) -> Self {
        self.table_map
            .as_object_mut()
            .unwrap()
            .remove(name)
            .unwrap();
        self
    }
}

/// Storage table manager
#[derive(Debug)]
pub struct JsonStorageTableManager {}

impl TableManager for JsonStorageTableManager {
    fn get_all() {
        unimplemented!();
    }

    fn push() {
        unimplemented!();
    }
}

/// Storage version
#[derive(Debug)]
pub struct JsonStorageVersion {
    pub(crate) value_map: Map<String, Value>,
}

impl Version for JsonStorageVersion {
    type TableDefinition = JsonStorageTableDefinition;

    fn add_table(mut self, name: &str) -> Self {
        self.value_map.insert(name.into(), Map::new().into());
        self
    }

    fn add_and_update_table<F>(self, name: &str, cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition,
    {
        self.add_table(name).update_table(name, cb)
    }

    fn remove_table(mut self, name: &str) -> Self {
        self.value_map.remove(name).unwrap();
        self
    }

    fn update_table<F>(mut self, name: &str, mut cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition,
    {
        let jstd = cb(JsonStorageTableDefinition {
            table_map: self.value_map[name].take(),
        });
        *self.value_map.get_mut(name).unwrap() = jstd.table_map;
        self
    }

    fn update_version(self) -> Self {
        unimplemented!();
    }
}
