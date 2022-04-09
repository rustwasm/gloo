//! A futures-based wrapper around indexed DB.
use futures::stream::Stream;
use gloo_events::{EventListener, EventListenerOptions};
use gloo_utils::window;
use std::{
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use wasm_bindgen::{prelude::*, throw_str, JsCast, UnwrapThrowExt};
use web_sys::{
    DomException, IdbDatabase, IdbFactory, IdbObjectStoreParameters, IdbOpenDbRequest, IdbRequest,
    IdbRequestReadyState, IdbTransaction, IdbTransactionMode, IdbVersionChangeEvent,
};

mod util;
pub use util::{StringList, StringListIter};
mod object_store;
pub use object_store::{
    CreateIndex, ObjectStoreDuringUpgrade, ObjectStoreReadOnly, ObjectStoreReadWrite, OpenCursor,
};
mod key;
pub use key::{IntoKeyPath, Key, KeyPath, Query};
mod transaction;
pub use transaction::{TransactionDuringUpgrade, TransactionReadOnly, TransactionReadWrite};
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
            .expect_throw("Database::open");

        // Listeners keep the closures alive unless dropped, in which case they are cleaned up.
        // Using `let _ = ...` would immediately drop the closure meaning it is not run.
        let _upgrade_listener = EventListener::new(&request, "upgradeneeded", {
            let request = request.clone();
            move |event| {
                let event = event
                    .dyn_ref::<IdbVersionChangeEvent>()
                    .expect_throw("IdbVersionChangeEvent dyn_into");
                let old_version = event.old_version() as u32;
                // newVersion is only null in a delete transation (which we know isn't happening
                // here)
                let new_version = event.new_version().unwrap_throw() as u32;
                let db = request
                    .result()
                    .expect_throw("IdbOpenDatabaseRequest::result")
                    .dyn_into::<IdbDatabase>()
                    .expect_throw("IdbDatabase dyn_into");

                // This seems to be the way to get a transation
                let transaction = request.transaction().expect_throw("request.transaction()");
                let transaction = TransactionDuringUpgrade::new(transaction);

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
        let inner = result
            .dyn_into::<IdbDatabase>()
            .expect_throw("dyn_into IdbDatabase");
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
    ) -> Result<TransactionReadWrite, errors::StartTransactionError> {
        let array = js_sys::Array::new();
        for store in stores {
            array.push(&JsValue::from_str(store.as_ref()));
        }
        self.transaction_inner(&array, IdbTransactionMode::Readwrite)
            .map(TransactionReadWrite::new)
    }

    /// Open a transaction with access to the given stores in "readonly" mode.
    pub fn transaction_readonly(
        &self,
        stores: &[impl AsRef<str>],
    ) -> Result<TransactionReadOnly, errors::StartTransactionError> {
        let array = js_sys::Array::new();
        for store in stores {
            array.push(&JsValue::from_str(store.as_ref()));
        }
        self.transaction_inner(&array, IdbTransactionMode::Readonly)
            .map(TransactionReadOnly::new)
    }

    fn transaction_inner(
        &self,
        stores: &JsValue,
        mode: IdbTransactionMode,
    ) -> Result<IdbTransaction, errors::StartTransactionError> {
        self.inner
            .transaction_with_str_sequence_and_mode(stores, mode)
            .map_err(errors::StartTransactionError::from)
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
    transaction: &'trans TransactionDuringUpgrade,
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
    pub fn delete_object_store(&self, name: &str) -> Result<(), errors::DeleteObjectStoreError> {
        self.db
            .inner
            .delete_object_store(name)
            .map_err(errors::DeleteObjectStoreError::from)
    }

    /// Get the database upgrade transaction.
    ///
    /// Use this method rather than `Database::start_*_transaction` if you want to rename or add indexes to
    /// already existing object stores.
    pub fn upgrade_transaction(&self) -> &'trans TransactionDuringUpgrade {
        self.transaction
    }
}

impl<'trans> Deref for DatabaseDuringUpgrade<'trans> {
    type Target = Database;

    fn deref(&self) -> &Database {
        &self.db
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

    /// The [key path] to be used by the new object store. If empty or not specified, the object
    /// store is created without a key path and uses [out-of-line keys]. You can also pass in an
    /// array as a `key_path`.
    ///
    /// [key path]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#key_path
    /// [out-of-line keys]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology#out-of-line_key
    pub fn key_path(mut self, key_path: impl IntoKeyPath) -> Self {
        self.params.key_path(Some(&key_path.into_jsvalue()));
        self
    }

    /// Actually create the object store using the configured builder.
    ///
    /// # Panics
    ///
    /// This function will panic if `auto_increment` is set to `false` (the default), and
    /// `key_path` is empty or not set.
    pub fn build(self) -> Result<ObjectStoreDuringUpgrade, errors::CreateObjectStoreError> {
        self.db
            .create_object_store_with_optional_parameters(self.name, &self.params)
            .map(ObjectStoreDuringUpgrade::new)
            .map_err(errors::CreateObjectStoreError::from)
    }
}

/// Wrapper around IdbRequest that implements `Future`.
struct Request {
    inner: IdbRequest,
    bubble_errors: bool,
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
}

impl Request {
    fn new(inner: IdbRequest, bubble_errors: bool) -> Self {
        Self {
            inner,
            bubble_errors,
            success_listener: None,
            error_listener: None,
        }
    }
}

impl Future for Request {
    type Output = Result<JsValue, DomException>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    self.success_listener = Some(EventListener::once(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| waker.wake()
                    }))
                } else {
                    throw_str("success_listener")
                }
                if self.error_listener.is_none() {
                    let opts = if self.bubble_errors {
                        EventListenerOptions::enable_prevent_default()
                    } else {
                        EventListenerOptions::default()
                    };
                    self.error_listener = Some(EventListener::once_with_options(
                        &self.inner,
                        "error",
                        opts,
                        {
                            let waker = cx.waker().clone();
                            let bubble_errors = self.bubble_errors;
                            move |event| {
                                waker.wake();
                                if !bubble_errors {
                                    event.prevent_default();
                                }
                            }
                        },
                    ))
                } else {
                    throw_str("error_listener")
                }
                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().expect_throw("get error") {
                    Poll::Ready(Err(error))
                } else {
                    // no error = success
                    Poll::Ready(Ok(self.inner.result().expect_throw("get result")))
                }
            }
            _ => throw_str("unknown ReadyState"),
        }
    }
}

/// Wrapper around IdbRequest that implements `Future`.
struct OpenDbRequest {
    inner: IdbOpenDbRequest,
    error_on_block: bool,
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
    blocked_listener: Option<EventListener>,
    blocked: Arc<AtomicBool>,
}

impl OpenDbRequest {
    fn new(inner: IdbOpenDbRequest, error_on_block: bool) -> Self {
        Self {
            inner,
            error_on_block,
            success_listener: None,
            error_listener: None,
            blocked_listener: None,
            blocked: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Future for OpenDbRequest {
    type Output = Result<JsValue, DomException>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.blocked.load(Ordering::SeqCst) {
            // return error
            return Poll::Ready(Err(DomException::new_with_message_and_name(
                "transaction would block",
                "TransactionWouldBlock",
            )
            .expect_throw("DomException")));
        }

        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    self.success_listener = Some(EventListener::once(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| waker.wake()
                    }))
                } else {
                    throw_str("success_listener")
                }
                if self.error_listener.is_none() {
                    self.error_listener = Some(EventListener::once(&self.inner, "error", {
                        let waker = cx.waker().clone();
                        move |_| waker.wake()
                    }))
                } else {
                    throw_str("error_listener")
                }
                if self.error_on_block {
                    if self.blocked_listener.is_none() {
                        self.blocked_listener = Some(EventListener::once(&self.inner, "blocked", {
                            let blocked = self.blocked.clone();
                            let waker = cx.waker().clone();
                            move |_| {
                                blocked.store(true, Ordering::SeqCst);
                                waker.wake();
                            }
                        }))
                    } else {
                        throw_str("blocked_lsitener")
                    }
                }
                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().expect_throw("error") {
                    Poll::Ready(Err(error))
                } else {
                    // no error = success
                    Poll::Ready(Ok(self.inner.result().expect_throw("result")))
                }
            }
            _ => throw_str("ready state"),
        }
    }
}

/// Wrapper for IDBRequest where the success callback is run multiple times.
// TODO If a task is woken up, does `wasm_bindgen_futures` try to progress the future in the same
// microtask or a separate one? This will impact whether I need to have space for more than one
// result at a time.
#[derive(Debug)]
pub struct StreamingRequest {
    inner: IdbRequest,
    bubble_errors: bool,
    success_listener: Option<EventListener>,
    error_listener: Option<EventListener>,
}

impl StreamingRequest {
    fn new(inner: IdbRequest, bubble_errors: bool) -> Self {
        Self {
            inner,
            bubble_errors,
            success_listener: None,
            error_listener: None,
        }
    }
}

impl Stream for StreamingRequest {
    type Item = Result<JsValue, DomException>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.ready_state() {
            IdbRequestReadyState::Pending => {
                if self.success_listener.is_none() {
                    // First call - setup
                    self.success_listener = Some(EventListener::new(&self.inner, "success", {
                        let waker = cx.waker().clone();
                        move |_| {
                            let waker = waker.clone();
                            waker.wake()
                        }
                    }));

                    // omit the error.is_none check to save a branch.
                    let opts = if self.bubble_errors {
                        EventListenerOptions::enable_prevent_default()
                    } else {
                        EventListenerOptions::default()
                    };
                    self.error_listener = Some(EventListener::new_with_options(
                        &self.inner,
                        "error",
                        opts,
                        {
                            let waker = cx.waker().clone();
                            let bubble_errors = self.bubble_errors;
                            move |event| {
                                let waker = waker.clone();
                                waker.wake();
                                if !bubble_errors {
                                    event.prevent_default();
                                }
                            }
                        },
                    ));
                }

                Poll::Pending
            }
            IdbRequestReadyState::Done => {
                if let Some(error) = self.inner.error().expect_throw("get error") {
                    Poll::Ready(Some(Err(error)))
                } else {
                    // no error = success
                    // if the result is null, there won't be any more entries (at least for
                    // IDBCursor, which I think is the only case a request is re-used)
                    let result = self.inner.result().expect_throw("get result");
                    if result.is_null() || result.is_undefined() {
                        Poll::Ready(None)
                    } else {
                        Poll::Ready(Some(Ok(result)))
                    }
                }
            }
            _ => throw_str("unreachable"),
        }
    }
}
