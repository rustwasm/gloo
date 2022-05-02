//! A futures-based wrapper around indexed DB.
use gloo_events::EventListener;
use gloo_utils::window;
use std::{future::Future, ops::Deref};
use wasm_bindgen::{prelude::*, JsCast, UnwrapThrowExt};
use web_sys::{
    IdbDatabase, IdbFactory, IdbObjectStoreParameters, IdbTransactionMode, IdbVersionChangeEvent,
};

mod util;
use util::UnreachableExt;
pub use util::{StringList, StringListIter};
mod request;
use request::{OpenDbRequest, Request, StreamingRequest};
mod object_store;
pub use object_store::{CursorOptions, IndexOptions, ObjectStore};
mod key;
pub use key::{IntoKeyPath, Key, KeyPath, Query};
mod transaction;
pub use transaction::Transaction;
mod index;
pub use index::Index;
mod cursor;
pub use cursor::{Cursor, CursorDirection, CursorStream, KeyCursor};
pub mod errors;

/// Marker type for read-only.
#[derive(Debug)]
pub enum ReadOnly {}
/// Marker type for read-write.
#[derive(Debug)]
pub enum ReadWrite {}
/// Marker type for upgrade.
#[derive(Debug)]
pub enum Upgrade {}

fn indexed_db() -> Option<IdbFactory> {
    window().indexed_db().ok().flatten()
}

/// Checks if indexed db is supported in the current context.
pub fn is_supported() -> bool {
    indexed_db().is_some()
}

/// Delete an indexed db database.
///
/// The database will be deleted whether the future is polled or not.
pub fn delete_database(
    name: &str,
    error_on_block: bool,
) -> impl Future<Output = Result<(), errors::DeleteDatabaseError>> {
    let request = indexed_db()
        .ok_or(errors::DeleteDatabaseError::IndexedDbUnsupported)
        .and_then(|factory| {
            factory
                .delete_database(name)
                .map_err(errors::DeleteDatabaseError::from)
        });

    async move {
        OpenDbRequest::new(request?, error_on_block)
            .await
            .map(|_| ())
            .map_err(errors::DeleteDatabaseError::from)
    }
}

/// An indexeddb database
#[derive(Debug)]
pub struct Database {
    inner: IdbDatabase,
}

impl Database {
    // TODO should we handle the 'block' event? It doesn't mean failure, just
    // that the future will not complete until other db instances are closed.
    // Maybe promote to an error?
    /// Open a database.
    ///
    ///  - The version must not be `0`.
    ///  - If error_on_block is `true`, then if the request would block, it instead returns
    ///    `OpenDatabaseError::WouldBlock`.
    ///
    /// If you need to use async functions in `upgrade_fn`, then use
    /// [`wasm_bindgen_futures::spawn_local`] to spawn a future. As long as at least one db
    /// operation is alive across all breakpoints, the transaction will not commit (it will
    /// auto-commit the first chance it gets).
    pub async fn open(
        name: &str,
        version: u32,
        mut upgrade_fn: impl FnMut(DatabaseDuringUpgrade) + 'static,
        error_on_block: bool,
    ) -> Result<Self, errors::OpenDatabaseError> {
        if version == 0 {
            return Err(errors::OpenDatabaseError::InvalidVersion);
        }
        let request = indexed_db()
            .ok_or(errors::OpenDatabaseError::IndexedDbUnsupported)?
            .open_with_u32(name, version)
            .unreachable_throw();

        // Listeners keep the closures alive unless dropped, in which case they are cleaned up.
        // Using `let _ = ...` would immediately drop the closure meaning it is not run.
        let _upgrade_listener = EventListener::new(&request, "upgradeneeded", {
            let request = request.clone();
            move |event| {
                let event = event.unchecked_ref::<IdbVersionChangeEvent>();
                let old_version = event.old_version() as u32;
                // newVersion is only null in a delete transation (which we know isn't happening
                // here)
                let new_version = event.new_version().unwrap_throw() as u32;
                let db = request
                    .result()
                    .unreachable_throw()
                    .unchecked_into::<IdbDatabase>();

                // Usually the transaction will be used to create a request, but in the db upgrade
                // case this is the only way to gain access to the upgrade transaction. We grab it
                // here to allow the user to do open existing object stores etc during upgrade.
                let transaction = request.transaction().expect_throw("request.transaction()");
                let transaction: Transaction<Upgrade> = Transaction::new(transaction);

                upgrade_fn(DatabaseDuringUpgrade {
                    old_version,
                    new_version,
                    db: Database { inner: db },
                    transaction: &transaction,
                });
            }
        });

        let result = OpenDbRequest::new(request, error_on_block)
            .await
            .map_err(errors::OpenDatabaseError::from)?;
        let inner = result.dyn_into::<IdbDatabase>().unreachable_throw();
        Ok(Database { inner })
    }

    /// Get the name of the db
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Get a list of all the object store names for the database.
    ///
    /// # Examples
    ///
    /// Does the db contain a "test" object store?
    /// ```no_run
    /// let contains_test: bool = db.object_store_names().contains("test").unwrap_throw();
    /// ```
    ///
    /// Collect names into a `Vec`..
    /// ```no_run
    /// db.object_store_names().into_iter().collect::<Vec<_>>()
    /// ```
    pub fn object_store_names(&self) -> StringList {
        StringList::new(self.inner.object_store_names())
    }

    /// Open a transaction with access to the given stores in "readwrite" mode.
    pub fn transaction_readwrite(
        &self,
        stores: &[impl AsRef<str>],
    ) -> Result<Transaction<ReadWrite>, errors::StartTransactionError> {
        self.transaction_inner(stores, IdbTransactionMode::Readwrite)
    }

    /// Open a transaction with access to the given stores in "readonly" mode.
    pub fn transaction_readonly(
        &self,
        stores: &[impl AsRef<str>],
    ) -> Result<Transaction<ReadOnly>, errors::StartTransactionError> {
        self.transaction_inner(stores, IdbTransactionMode::Readonly)
    }

    fn transaction_inner<Ty>(
        &self,
        stores: &[impl AsRef<str>],
        mode: IdbTransactionMode,
    ) -> Result<Transaction<Ty>, errors::StartTransactionError> {
        let array = js_sys::Array::new();
        for store in stores {
            array.push(&JsValue::from_str(store.as_ref()));
        }
        self.inner
            .transaction_with_str_sequence_and_mode(&array, mode)
            .map_err(errors::StartTransactionError::from)
            .map(Transaction::new)
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        self.inner.close()
    }
}

/// Provides access to the database during an update event.
///
/// Use this object to create/delete object stores and indexes. You can also get access to the
/// underlying DB, but note that any transactions created here will run after the database upgrade
/// has completed.
#[derive(Debug)]
pub struct DatabaseDuringUpgrade<'trans> {
    old_version: u32,
    new_version: u32,
    db: Database,
    transaction: &'trans Transaction<Upgrade>,
}

impl<'trans> DatabaseDuringUpgrade<'trans> {
    // these methods exist to make the values read-only.
    /// The database version at the start of this upgrade.
    pub fn old_version(&self) -> u32 {
        self.old_version
    }

    /// The database version at the end of this upgrade.
    pub fn new_version(&self) -> u32 {
        self.new_version
    }

    /// Create a new object store in the database
    ///
    /// # Panics
    ///
    /// This function will panic if `auto_increment` is set to `false` (the default), and
    /// `key_path` is empty or not set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// db.create_object_store("test")
    ///     .auto_increment(false)
    ///     .key_path("key.path")
    ///     .build()
    /// ```
    pub fn create_object_store<'a>(
        &'a self,
        name: &'a str,
        opts: ObjectStoreOptions,
    ) -> Result<ObjectStore<Upgrade>, errors::CreateObjectStoreError> {
        self.db
            .inner
            .create_object_store_with_optional_parameters(name, &opts.inner)
            .map(ObjectStore::new)
            .map_err(errors::CreateObjectStoreError::from)
    }
    /// Delete an object store from the database
    ///
    /// Returns `true` if an object store was deleted or `false` if no object store with
    /// that name existed. Errors if the database has been deleted since the update started.
    pub fn delete_object_store(&self, name: &str) -> Result<(), errors::DeleteObjectStoreError> {
        self.db
            .inner
            .delete_object_store(name)
            .map_err(errors::DeleteObjectStoreError::from)
    }

    /// Get the transaction this database upgrade is running in.
    ///
    /// Use this method rather than `Database::start_*_transaction` if you want to rename or add indexes to
    /// already existing object stores.
    pub fn transaction(&self) -> &'trans Transaction<Upgrade> {
        self.transaction
    }
}

impl<'trans> Deref for DatabaseDuringUpgrade<'trans> {
    type Target = Database;

    fn deref(&self) -> &Database {
        &self.db
    }
}

/// Possible objects when creating an object store
#[derive(Debug)]
pub struct ObjectStoreOptions {
    inner: IdbObjectStoreParameters,
}

impl ObjectStoreOptions {
    /// The default options
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    /// If `true`, the object store has a
    /// [key generator](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_generator).
    /// Defaults to `false`.
    pub fn auto_increment(mut self, auto_increment: bool) -> Self {
        self.inner.auto_increment(auto_increment);
        self
    }

    /// The [key path] to be used by the new object store. If empty or not specified, the object
    /// store is created without a key path and uses [out-of-line keys]. You can also pass in an
    /// array as a `key_path`.
    ///
    /// [key path]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_path
    /// [out-of-line keys]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#out-of-line_key
    pub fn key_path(mut self, key_path: impl IntoKeyPath) -> Self {
        self.inner.key_path(Some(&key_path.into_jsvalue()));
        self
    }
}

impl Default for ObjectStoreOptions {
    fn default() -> Self {
        Self::new()
    }
}
