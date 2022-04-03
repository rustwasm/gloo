//! A futures-based wrapper around indexed DB.
use std::ops::Index;

use futures::StreamExt;
use gloo_events::EventListener;
use gloo_utils::window;
use wasm_bindgen::{prelude::*, throw_str, JsCast};
use web_sys::{
    DomException, DomStringList, IdbDatabase, IdbFactory, IdbObjectStore, IdbObjectStoreParameters,
    IdbOpenDbRequest, IdbRequest, IdbVersionChangeEvent,
};

/// This crate's result type.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

fn indexed_db() -> Option<IdbFactory> {
    window().indexed_db().ok().flatten()
}

/// Checks if indexed db is supported in the current context.
pub fn indexed_db_supported() -> bool {
    indexed_db().is_some()
}

/// Delete an indexed db database.
pub async fn delete_db(name: &str) {
    let db_request = indexed_db()
        .unwrap_throw()
        .delete_database(name)
        .unwrap_throw();

    let (mut send, mut recv) = futures_channel::mpsc::channel(1);
    let _success_listener = EventListener::new(&db_request, "success", {
        let mut send = send.clone();
        move |_| {
            send.try_send(Ok(())).expect_throw("try_send");
        }
    });

    let _error_listener = EventListener::new(&db_request, "error", move |_| {
        send.try_send(Err(())).expect_throw("try_send")
    });

    recv.next().await.unwrap_throw().expect_throw("delete_db")
}

/// An indexeddb database
#[derive(Debug)]
pub struct Db {
    inner: IdbDatabase,
}

impl Db {
    // TODO should we handle the 'block' event? It doesn't mean failure, just
    // that the future will not complete until other db instances are closed.
    // Maybe promote to an error?
    /// Open a database
    ///
    /// # Panics
    ///
    /// Will panic if `version` is `0`.
    pub async fn open(
        name: &str,
        version: u32,
        mut upgrade_fn: impl FnMut(DbUpgrade) + 'static,
    ) -> Result<Db, Error> {
        if version == 0 {
            throw_str("version must be at least 1");
        }
        let db_request = indexed_db()
            .ok_or(Error::IndexedDbNotFound)?
            .open_with_u32(name, version)
            .expect_throw("Db::open");

        // Listeners keep the closures alive unless dropped, in which case they are cleaned up.
        // Using `let _ = ...` would immediately drop the closure meaning it is not run.
        let _upgrade_listener = EventListener::new(&db_request, "upgradeneeded", move |event| {
            let event = event
                .dyn_ref::<IdbVersionChangeEvent>()
                .expect_throw("IdbVersionChangeEvent dyn_into");
            let old_version = event.old_version() as u32;
            // newVersion is not optional on MDN - I think this is an IDL inaccuracy.
            let new_version = event.new_version().expect_throw("new_version") as u32;
            let db = event
                .target()
                .expect_throw("Event::target")
                .dyn_into::<IdbOpenDbRequest>()
                .expect_throw("IdbOpenDbRequest dyn_into")
                .result()
                .expect_throw("IdbOpenDbRequest::result")
                .dyn_into::<IdbDatabase>()
                .expect_throw("IdbDatabase dyn_into");

            upgrade_fn(DbUpgrade {
                old_version,
                new_version,
                db: Db { inner: db },
            })
        });

        // Exactly one message should be sent on the channel, meaning there should never be back-pressure.
        // So errors should never happen when sending.
        let (mut send, mut recv) = futures_channel::mpsc::channel(1);
        let _success_listener = EventListener::new(&db_request, "success", {
            let mut send = send.clone();
            move |event| {
                let db = event
                    .target()
                    .expect_throw("Event::target")
                    .dyn_into::<IdbOpenDbRequest>()
                    .expect_throw("IdbOpenDbRequest::dyn_into")
                    .result()
                    .expect_throw("IdbOpenDbRequest::result")
                    .dyn_into::<IdbDatabase>()
                    .expect_throw("IdbDatabase::dyn_into");
                send.try_send(Some(db)).expect_throw("try_send");
            }
        });

        let _error_listener = EventListener::new(&db_request, "error", move |_| {
            send.try_send(None).expect_throw("try_send")
        });

        // After this await, either error or success will have fired (this also means upgrading will have taken place)
        match recv.next().await.flatten() {
            Some(db) => Ok(Db { inner: db }),
            None => Err(Error::OpeningDb),
        }
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
    pub fn object_store_names(&self) -> DomStringList {
        self.inner.object_store_names()
    }

    /// Copy the object store names for this database into a `Vec`.
    pub fn object_store_names_vec(&self) -> Vec<String> {
        let raw = self.object_store_names();
        let mut names = vec![];
        for i in 0..raw.length() {
            names.push(raw.get(i).unwrap_throw());
        }
        names
    }
}

/// Provides access to the database during an update event.
///
/// Use this object to create/delete object stores and indexes.
#[derive(Debug)]
pub struct DbUpgrade {
    /// The version we are upgrading from
    pub old_version: u32,
    /// The version we are upgrading to
    pub new_version: u32,
    db: Db,
}

impl DbUpgrade {
    /// Create a new object store in the database
    ///
    /// # Example
    ///
    /// ```no_run
    /// db.create_object_store("test")
    ///     .auto_increment(false)
    ///     .key_path("key.path")
    ///     .build()
    /// ```
    pub fn create_object_store<'a>(&'a self, name: &'a str) -> CreateObjectStore<'a> {
        CreateObjectStore {
            name,
            params: IdbObjectStoreParameters::new(),
            db: &self.db.inner,
        }
    }
    /// Delete an object store from the database
    ///
    /// Returns `true` if an object store was deleted or `false` if no object store with
    /// that name existed. Errors if the database has been deleted since the update started.
    pub fn delete_object_store(&self, name: &str) -> Result<bool> {
        match self.db.inner.delete_object_store(name) {
            Ok(()) => Ok(true),
            Err(error) => {
                let error = error.dyn_into::<DomException>().unwrap_throw();
                match error.name().as_str() {
                    "TransactionInactiveError" => Err(Error::DbRemoved),
                    "NotFoundError" => Ok(false),
                    e => throw_str(&format!("unexpected error {}", e)),
                }
            }
        }
    }
}

/// Builder struct to create object stores
#[derive(Debug)]
pub struct CreateObjectStore<'a> {
    name: &'a str,
    params: IdbObjectStoreParameters,
    db: &'a IdbDatabase,
}

impl<'a> CreateObjectStore<'a> {
    /// If `true`, the object store has a
    /// [key generator](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_generator).
    /// Defaults to `false`.
    pub fn auto_increment(mut self, auto_increment: bool) -> Self {
        self.params.auto_increment(auto_increment);
        self
    }

    /// The [key path](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_path)
    /// to be used by the new object store. If empty or not specified, the object store is created without a key path and uses
    /// [out-of-line keys](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#out-of-line_key).
    /// You can also pass in an array as a `key_path`.
    pub fn key_path(mut self, key_path: impl KeyPath) -> Self {
        self.params.key_path(Some(&key_path.into_jsvalue()));
        self
    }

    /// Actually create the object store using the configured builder.
    ///
    /// # Panics
    ///
    /// This function will panic if `auto_increment` is set to `false` (the default), and `key_path` is empty or not set.
    pub fn build(self) -> Result<IdbObjectStore> {
        self.db
            .create_object_store_with_optional_parameters(self.name, &self.params)
            .map_err(|error| {
                let error = error.dyn_into::<DomException>().unwrap_throw();
                // TODO MDN is a little vague here - so it would be worth revisiting and nailing down
                // exact behavior
                match error.name().as_str() {
                    "TransactionInactiveError" => Error::DbRemoved,
                    "ConstraintError" => Error::ObjectStoreAlreadyExists,
                    "InvalidAccessError" => {
                        throw_str("auto_increment is true and key_path is empty")
                    }
                    e => throw_str(&format!("unexpected error {}", e)),
                }
            })
    }
}

mod sealed {
    pub trait Sealed {}

    impl<'a> Sealed for &'a str {}
    impl<'a, T> Sealed for &'a [T] where T: AsRef<str> + 'a {}
}

/// A trait for types that can be used as a key path when creating an object store.
///
/// Types allowed are either a string or an array of strings. An empty string is
/// equivalent to not setting the key path.
pub trait KeyPath: sealed::Sealed {
    /// Internal - please ignore
    ///
    /// Converts self into a value to use as the keyPath (must be a JsValue)
    fn into_jsvalue(self) -> JsValue;
}

impl<'a> KeyPath for &'a str {
    fn into_jsvalue(self) -> JsValue {
        JsValue::from(self)
    }
}

impl<'a, T> KeyPath for &'a [T]
where
    T: AsRef<str> + 'a,
{
    fn into_jsvalue(self) -> JsValue {
        let arr = js_sys::Array::new();
        for i in 0..self.len() {
            arr.push(&JsValue::from(self[i].as_ref()));
        }
        JsValue::from(arr)
    }
}

/// Represents an error using indexeddb.
#[derive(Debug)]
pub enum Error {
    /// An error occurred while opening a database.
    OpeningDb,
    /// The current context does not support indexed db.
    IndexedDbNotFound,
    /// Attempted to create an object store on a database that had already
    /// been deleted.
    DbRemoved,
    /// An object store already exists with the same name as one being
    /// created (case sensitive).
    ObjectStoreAlreadyExists,
    /// A custom error - not used by the crate but allows users to pass their
    /// own errors through in certain circumstances.
    Custom(Box<dyn std::error::Error + 'static>),
}
