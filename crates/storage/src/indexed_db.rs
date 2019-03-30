use crate::{Index, Storage, TableDefinition, TableManager, Version};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbDatabase, IdbFactory, IdbIndexParameters, IdbObjectStore, IdbOpenDbOptions};

/// IndexedDB
#[derive(Debug)]
pub struct IndexedDb {
    database_name: String,
    factory: IdbFactory,
}

impl IndexedDb {
    /// Creates a new instance for database `name`.
    pub fn new<I>(name: I) -> Self
    where
        I: Into<String>,
    {
        Self {
            database_name: name.into(),
            factory: web_sys::window()
                .unwrap_throw()
                .indexed_db()
                .unwrap_throw()
                .unwrap_throw(),
        }
    }
}

impl Storage for IndexedDb {
    type TableManager = IndexedDbTableManager;
    type Version = IndexedDbVersion;

    fn add_version<F, I>(&mut self, version: I, cb: F)
    where
        F: FnOnce(Self::Version) -> Self::Version + 'static,
        I: Into<f64>,
    {
        let request = self
            .factory
            .open_with_idb_open_db_options(
                &self.database_name,
                IdbOpenDbOptions::new().version(version.into()),
            )
            .unwrap_throw();
        let result = request.result();
        let closure = Closure::once(move || {
            let database = result.unwrap_throw().unchecked_into::<IdbDatabase>();
            cb(IndexedDbVersion { database })
        });
        let func = closure.as_ref().unchecked_ref::<js_sys::Function>();
        request.set_onupgradeneeded(Some(func));
    }

    fn delete(&mut self) {
        self.factory
            .delete_database(&self.database_name)
            .unwrap_throw();
    }

    fn name(&self) -> &str {
        &self.database_name
    }

    fn table_manager(&mut self, _: &str) -> Self::TableManager {
        unimplemented!();
    }

    fn transaction(&mut self) {
        unimplemented!();
    }
}

/// IndexedDB table definition
#[derive(Debug)]
pub struct IndexedDbTableDefinition {
    object_store: IdbObjectStore,
}

impl TableDefinition for IndexedDbTableDefinition {
    fn add_row_with_index(self, name: &str, index: Index) -> Self {
        let mut params = IdbIndexParameters::new();
        match index {
            Index::MultiEntry => {
                params.multi_entry(true);
            }
            Index::Unique => {
                params.unique(true);
            }
            _ => {}
        };
        self.object_store
            .create_index_with_str_and_optional_parameters(name, name, &params)
            .unwrap_throw();
        self
    }

    fn remove_old_row(self, name: &str) -> Self {
        self.object_store.delete(&name.into()).unwrap_throw();;
        self
    }
}

/// IndexedDB table manager
#[derive(Debug)]
pub struct IndexedDbTableManager();

impl TableManager for IndexedDbTableManager {
    fn get_all() {
        unimplemented!();
    }

    fn push() {
        unimplemented!();
    }
}

/// IndexedDB version
#[wasm_bindgen]
#[derive(Debug)]
pub struct IndexedDbVersion {
    database: web_sys::IdbDatabase,
}

impl Version for IndexedDbVersion {
    type TableDefinition = IndexedDbTableDefinition;

    fn add_table(self, name: &str) -> Self {
        self.database.create_object_store(name).unwrap_throw();
        self
    }

    fn add_and_update_table<F>(self, name: &str, cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition,
    {
        self.add_table(name).update_table(name, cb)
    }

    fn remove_table(self, name: &str) -> Self {
        self.database.delete_object_store(name).unwrap_throw();
        self
    }

    fn update_table<F>(self, name: &str, mut cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition,
    {
        cb(IndexedDbTableDefinition {
            object_store: self
                .database
                .transaction_with_str(name)
                .unwrap_throw()
                .object_store(name)
                .unwrap(),
        });
        self
    }

    fn update_version(self) -> Self {
        unimplemented!();
    }
}
