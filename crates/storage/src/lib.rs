//! AA

#![deny(missing_docs, missing_debug_implementations)]

#[cfg(feature = "indexed-db")]
pub(crate) mod indexed_db;
#[cfg(feature = "json-storage")]
pub(crate) mod json_storage;

#[cfg(feature = "indexed-db")]
pub use indexed_db::*;
#[cfg(feature = "json-storage")]
pub use json_storage::*;

/// Row index.
#[derive(Debug)]
pub enum Index {
    /// Auto increment
    AutoIncrement,
    /// Compound
    Compound,
    /// Multi entry
    MultiEntry,
    /// Unique
    Unique,
}

/// Storage.
pub trait Storage {
    /// Table manager.
    type TableManager;
    /// Version.
    type Version;

    /// Adds a new version into the current database.
    fn add_version<F, I>(&mut self, version: I, cb: F)
    where
        F: FnOnce(Self::Version) -> Self::Version + 'static,
        I: Into<f64>;

    /// Delets this database.
    fn delete(&mut self);

    /// The name of this database.
    fn name(&self) -> &str;

    /// Data manipulation for the most recent version.
    fn table_manager(&mut self, table_name: &str) -> Self::TableManager;

    /// Transaction
    fn transaction(&mut self);
}

/// Table definition.
///
/// You don't need to define an ordinary row because they are automatically included
/// in a DML operation.
pub trait TableDefinition {
    /// Defines a new row with index.
    fn add_row_with_index(self, name: &str, index: Index) -> Self;

    /// Removes a row from previous versions.
    fn remove_old_row(self, name: &str) -> Self;
}

/// Table manager.
///
/// Provides methods for DML (Data Manipulation Language).
pub trait TableManager {
    /// Gets all records.
    fn get_all();
    /// Pushes a new record.
    fn push();
}

/// Defines the state of a database.
pub trait Version {
    /// Table definition
    type TableDefinition;

    /// Adds a new table `name`.
    fn add_table(self, name: &str) -> Self;

    /// Adds and updates the table `name`.
    fn add_and_update_table<F>(self, name: &str, cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition;

    /// Removes a table `name`
    fn remove_table(self, name: &str) -> Self;

    /// Updates the table `name`.
    fn update_table<F>(self, name: &str, cb: F) -> Self
    where
        F: FnMut(Self::TableDefinition) -> Self::TableDefinition;

    /// Updates this version against the previous version.
    fn update_version(self) -> Self;
}
