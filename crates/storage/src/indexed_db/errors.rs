//! Various error types for operations in `indexed_db`.
//!
//! Some errors aren't handled explicitaly - these are the ones that we prevent using the Rust type
//! system.
use thiserror::Error;

/// Errors that can occur when opening a database.
#[derive(Debug, Error)]
pub enum OpenDatabaseError {
    /// Could not get the indexedDB singleton
    #[error("indexeddb appears to be unsupported on this platform")]
    IndexedDbUnsupported,
    /// The database version was set to 0
    #[error("the database version was set to 0")]
    InvalidVersion,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(OpenDatabaseError {
    "TypeError" => InvalidVersion,
});

/// Errors that can occur when deleting a database.
#[derive(Debug, Error)]
pub enum DeleteDatabaseError {
    /// Could not get the indexedDB singleton
    #[error("indexeddb appears to be unsupported on this platform")]
    IndexedDbUnsupported,
    /// Another connection is blocking us
    #[error("another connection is blocking us")]
    WouldBlock,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(DeleteDatabaseError {
    "TransactionWouldBlock" => WouldBlock,
});

/// Errors that can occur when creating an object store.
#[derive(Debug, Error)]
pub enum CreateObjectStoreError {
    /// Could not get the indexedDB singleton
    #[error("indexeddb appears to be unsupported on this platform")]
    IndexedDbUnsupported,
    /// The upgrade transaction has already finished
    #[error("the upgrade transaction has already finished")]
    TransactionInactive,
    /// A store with the same name already exists
    #[error("a store with the same name already exists")]
    StoreAlreadyExists,
    /// Trying to set `auto_increment = true` with an empty `key_path`
    #[error("trying to set `auto_increment = true` with an empty `key_path`")]
    InvalidConfig,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(CreateObjectStoreError {
    "TransactionInactiveError" => TransactionInactive,
    "ConstraintError" => StoreAlreadyExists,
    "InvalidAccessError" => InvalidConfig,
});

/// Errors that can occur when deleting an object store
#[derive(Debug, Error)]
pub enum DeleteObjectStoreError {
    /// The upgrade transaction has finished
    #[error("the upgrade transaction has finished")]
    TransactionInactive,
    /// Tried to delete an object store that doesn't exist
    #[error("tried to delete an object store that doesn't exist")]
    ObjectStoreNotFound,
    /// Unexpected error
    #[error("unexepcted error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(DeleteObjectStoreError {
    "TransactionInactiveError" => TransactionInactive,
    "NotFoundError" => ObjectStoreNotFound
});

/// An error opening an object store from a transaction.
#[derive(Debug, Error)]
pub enum ObjectStoreError {
    /// No object store with the given name was found (case sensitive)
    #[error("no object store with the given name was found (case sensitive)")]
    NotFound,
    /// The object store was deleted or moved, or the transaction has finished
    #[error("the object store was deleted or moved, or the transaction has finished")]
    InvalidState,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(ObjectStoreError {
    "NotFoundError" => NotFound,
    "InvalidStateError" => InvalidState,
});

/// An error setting the name of an object store.
#[derive(Debug, Error)]
pub enum SetNameError {
    /// an object store with the given name already exists.
    #[error("an object store with the given name already exists")]
    StoreWithNameExists,
    /// Cannot change the name because the transaction has finished or been cancelled
    #[error("cannot change the name because the transaction has finished or been cancelled")]
    TransactionInactive,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

/// An error creating a new index on an object store
#[derive(Debug, Error)]
pub enum CreateIndexError {
    /// An index with the given name already exists (index names are case sensitive)
    #[error("an index with the given name already exists (case sensitive)")]
    IndexWithNameExists,
    /// Cannot set `multi_entry(true)` when `key_path` is a sequence
    #[error("cannot set `multi_entry(true)` when `key_path` is a sequence")]
    SequenceMultiEntry,
    /// The object store we are trying to create an index for has been deleted
    #[error("the object store we are trying to create an index for has been deleted")]
    ObjectStoreDeleted,
    /// The key path given to create_index isn't a valid key path
    #[error("the key path given to create_index isn't a valid key path")]
    InvalidKeyPath,
    /// The upgrade transaction had finished before the index could be created
    #[error("the upgrade transaction had finished before the index could be created")]
    TransactionInactive,
    /// Unexpected error creating index
    #[error("unexpected error creating index: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(CreateIndexError {
    "ConstraintError" => IndexWithNameExists,
    "InvalidAccessError" => SequenceMultiEntry,
    "InvalidStateError" => ObjectStoreDeleted,
    "SyntaxError" => InvalidKeyPath,
    "TransactionInactiveError" => TransactionInactive,
});

/// An error deleting an index
#[derive(Debug, Error)]
pub enum DeleteIndexError {
    /// The upgrade transaction had finished before the index could be created
    #[error("the upgrade transaction had finished before the index could be created")]
    TransactionInactive,
    /// No index was found with the given name
    #[error("no index was found with the given name")]
    NotFound,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(DeleteIndexError {
    "TransactionInactiveError" => TransactionInactive,
    "NotFoundError" => NotFound,
});

/// An error deleting an index
#[derive(Debug, Error)]
pub enum StartTransactionError {
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(StartTransactionError {});

/// An error adding an object to the store.
#[derive(Debug, Error)]
pub enum AddError {
    /// Tried to add an object within a transaction that has finished
    #[error("tried to add an object within a transaction that has finished")]
    TransactionInactive,
    /// Tried to add an invalid object - see [MDN](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/add#dataerror)
    #[error(
        "tried to add an invalid object - see \
        [MDN](https://developer.mozilla.org/en-US/docs/Web/API/IDBObjectStore/add#dataerror)"
    )]
    InvalidData,
    /// The object store was deleted or moved
    #[error("the object store was deleted or moved")]
    StoreNotFound,
    /// The structural clone of the object to be added failed
    #[error("the structural clone of the object to be added failed")]
    CloneFailed,
    /// Adding this object would violate a unique constraint
    #[error("adding this object would violate a unique constraint")]
    ConstraintViolated,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(AddError {
    "TransactionInactiveError" => TransactionInactive,
    "DataError" => InvalidData,
    "InvalidStateError" => StoreNotFound,
    "DataCloneError" => CloneFailed,
    "ConstraintError" => ConstraintViolated,
});

/// An error when deleting all records from an object store
#[derive(Debug, Error)]
pub enum ClearError {
    /// Tried to add an object within a transaction that has finished
    #[error("tried to add an object within a transaction that has finished")]
    TransactionInactive,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(ClearError {
    "TransactionInactiveError" => TransactionInactive,
});

/// An error when deleting all records from an object store
#[derive(Debug, Error)]
pub enum CountError {
    /// The object store was deleted or moved
    #[error("the object store was deleted or moved")]
    StoreNotFound,
    /// Tried to count objects within a transaction that has finished
    #[error("tried to count objects within a transaction that has finished")]
    TransactionInactive,
    /// The key or key range passed as a query was invalid
    ///
    /// The query option isn't implemented yet so this error will currently never occur.
    #[error("the key or key range passed as a query was invalid")]
    KeyRangeInvalid,
    /// The number of records returned is greater than the maximum safe integer, meaning it may not
    /// be accurate (not all numbers are representable as `f64`, JavaScript's number type).
    #[error("returned count greater than the maximum safe integer (2^53-1)")]
    CountTooBig,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(CountError {
    "InvalidStateError" => StoreNotFound,
    "TransactionInactiveError" => TransactionInactive,
    "DataError" => KeyRangeInvalid,
});

/// An error when deleting objects from an objecct store
#[derive(Debug, Error)]
pub enum DeleteError {
    /// Tried to delete objects within a transaction that has finished
    #[error("tried to delete objects within a transaction that has finished")]
    TransactionInactive,
    /// The object store was deleted or moved
    #[error("the object store was deleted or moved")]
    StoreNotFound,
    /// The given key was not a valid key
    ///
    /// This should only happen in edge cases.
    #[error("the given key was not a valid key")]
    InvalidKey,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(DeleteError {
    "TransactionInactiveError" => TransactionInactive,
    "InvalidStateError" => StoreNotFound,
    "DataError" => InvalidKey,
});

/// An error when deleting objects from an objecct store
#[derive(Debug, Error)]
pub enum GetError {
    /// Tried to get an object within a transaction that has finished
    #[error("tried to get an object within a transaction that has finished")]
    TransactionInactive,
    /// The object store was deleted or moved
    #[error("the object store was deleted or moved")]
    StoreNotFound,
    /// The given key was not a valid key
    ///
    /// This should only happen in edge cases.
    #[error("the given key was not a valid key")]
    InvalidKey,
    /// Could not deserialize results
    #[error("could not deserialize results")]
    Deserialize(serde_wasm_bindgen::Error),
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(GetError {
    "TransactionInactiveError" => TransactionInactive,
    "InvalidStateError" => StoreNotFound,
    "DataError" => InvalidKey,
});

/// An error when deleting objects from an objecct store
#[derive(Debug, Error)]
pub enum CursorError {
    /// Tried to open an cursor within a transaction that has finished
    #[error("tried to open a cursor within a transaction that has finished")]
    TransactionInactive,
    /// The object store was deleted or moved
    #[error("the object store was deleted or moved")]
    StoreNotFound,
    /// The given key was not a valid key
    ///
    /// This should only happen in edge cases, and only when a query is used.
    #[error("the given key was not a valid key")]
    InvalidKey,
    /// Could not deserialize results
    #[error("could not deserialize results")]
    Deserialize(serde_wasm_bindgen::Error),
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(CursorError {
    "TransactionInactiveError" => TransactionInactive,
    "InvalidStateError" => StoreNotFound,
    "DataError" => InvalidKey,
});

/// An error when deleting objects from an objecct store
#[derive(Debug, Error)]
pub enum KeyRangeError {
    /// Either upper < lower, upper = lower and at least one is closed, or upper or lower not a
    /// valid key
    // TODO options here are to panic on invalid ranges or check the conditions ourselves.
    #[error("upper < lower, upper = lower and one is closed, or upper or lower not a valid key")]
    InvalidParams,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(KeyRangeError {
    "DataError" => InvalidParams,
});

/// An error when deleting objects from an objecct store
#[derive(Debug, Error)]
pub enum AdvanceError {
    /// The amount was not a positive integer (should be unreachable)
    #[error("The amount was not a positive integer (should be unreachable)")]
    InvalidParams,
    /// The transaction this cursor is attached to is no longer active
    #[error("the transaction this cursor is attached to is no longer active")]
    TransactionInactive,
    /// The cursor is past the end of the object store (should be unreachable)
    #[error("the cursor is past the end of the object store (should be unreachable)")]
    PastEnd,
    /// Unexpected error
    #[error("unexpected error: {0}")]
    Unexpected(String),
}

error_from_jsvalue!(AdvanceError {
    "TypeError" => InvalidParams,
    "TransactionInactiveError" => TransactionInactive,
    "InvalidStateError" => PastEnd,
});

// key conversions

/// Tried to use a f64 NaN as a key
#[derive(Debug, Error)]
#[error("tried to use a f64 NaN as a key")]
pub struct NumberIsNan;
