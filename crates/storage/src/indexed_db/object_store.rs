// NOTE: all transaction operations must be started on the *same tick* (i.e. not in an async block)
// otherwise with transaction will auto-commit before the operation is started.
use super::{
    errors, CursorDirection, CursorStream, Index, IntoKeyPath, Key, KeyPath, Query, Request,
    StreamingRequest, StringList, TransactionDuringUpgrade,
};
use futures::{
    future::{ready, Either},
    FutureExt,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{convert::TryFrom, future::Future, ops::Deref};
use wasm_bindgen::{prelude::*, throw_str, JsCast, UnwrapThrowExt};
use web_sys::{IdbIndexParameters, IdbObjectStore};

/// An object store during a database upgrade.
///
/// Note that object stores are always sorted with their key values ascending.
#[derive(Debug)]
pub struct ObjectStoreDuringUpgrade {
    inner: ObjectStoreReadWrite,
}

impl ObjectStoreDuringUpgrade {
    pub(crate) fn new(inner: IdbObjectStore) -> Self {
        Self {
            inner: ObjectStoreReadWrite::new(inner),
        }
    }

    /// Changes the object store's name.
    ///
    /// # Panics
    ///
    /// Currently this method will panic on error. If/when [this wasm-bindgen patch](https://github.com/rustwasm/wasm-bindgen/pull/2852)
    /// lands errors will be returned instead. Because of the return type, the change will be
    /// backwards-compatible.
    pub fn set_name(&self, new_name: &str) -> Result<(), errors::SetNameError> {
        self.raw().set_name(new_name);
        Ok(())
    }

    /// Create a new index for the object store.
    pub fn create_index<'b, K: IntoKeyPath>(
        &'b self,
        name: &'b str,
        key_path: K,
    ) -> CreateIndex<'b, K> {
        CreateIndex {
            store: self.raw(),
            name,
            key_path,
            params: IdbIndexParameters::new(),
        }
    }

    /// Delete the index with the given name.
    pub fn delete_index(&self, name: &str) -> Result<(), errors::DeleteIndexError> {
        self.raw()
            .delete_index(name)
            .map_err(errors::DeleteIndexError::from)
    }
}

impl Deref for ObjectStoreDuringUpgrade {
    type Target = ObjectStoreReadWrite;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A builder object for creating an index.
///
/// Once you have set the options you require, call `build` to run the operation.
#[derive(Debug)]
pub struct CreateIndex<'a, K: IntoKeyPath> {
    store: &'a IdbObjectStore,
    name: &'a str,
    key_path: K,
    params: IdbIndexParameters,
}

impl<'a, K: IntoKeyPath> CreateIndex<'a, K> {
    /// If `true`, the index will not allow duplicate values for a single key.
    pub fn unique(mut self, yes: bool) -> Self {
        self.params.unique(yes);
        self
    }

    /// If `true`, the index will add an entry in the index for each array element when the
    /// *keyPath* resolves to an `Array`. If `false`, it will add one single entry containing the
    /// `Array`.
    pub fn multi_entry(mut self, yes: bool) -> Self {
        self.params.multi_entry(yes);
        self
    }

    // TODO potentially add `locale` in the future.

    ///
    /// Note that object stores are always sorted with their key values ascending.
    /// Run the create index operation.
    ///
    /// If you don't call this method, then the index won't be created.
    pub fn build(self) -> Result<Index, errors::CreateIndexError> {
        self.store
            .create_index_with_str_sequence_and_optional_parameters(
                self.name,
                &self.key_path.into_jsvalue(),
                &self.params,
            )
            .map(Index::new)
            .map_err(errors::CreateIndexError::from)
    }
}

/// An object store during a `readwrite` transaction.
///
/// Note that object stores are always sorted with their key values ascending.
#[derive(Debug)]
pub struct ObjectStoreReadWrite {
    inner: ObjectStoreReadOnly,
}

impl ObjectStoreReadWrite {
    pub(crate) fn new(inner: IdbObjectStore) -> Self {
        Self {
            inner: ObjectStoreReadOnly::new(inner),
        }
    }

    /// Add an object to the database.
    ///
    /// This method returns a future, but always tries to add the object irrespective of whether
    /// the future is ever polled. If `bubble_errors = true` any errors returned here will also
    /// cause the transaction to abort.
    pub fn add_raw(
        &self,
        value: &JsValue,
        key: Option<Key>,
        bubble_errors: bool,
    ) -> impl Future<Output = Result<(), errors::AddError>> {
        let request = if let Some(key) = key {
            self.raw().add_with_key(value, &key.0)
        } else {
            self.raw().add(value)
        };

        async move {
            let request = match request {
                Ok(request) => request,
                Err(e) => return Err(errors::AddError::from(e)),
            };
            match Request::new(request, bubble_errors).await {
                Ok(_) => Ok(()),
                Err(e) => Err(errors::AddError::from(e)),
            }
        }
    }

    /// Add an arbitrary object to the database using serde to serialize it to a JsValue.
    pub fn add(
        &self,
        value: &(impl Serialize + ?Sized),
        key: Option<Key>,
        bubble_errors: bool,
    ) -> impl Future<Output = Result<(), errors::AddError>> {
        // TODO handle errors
        let value = serde_wasm_bindgen::to_value(value).unwrap();
        let request = self.add_raw(&value, key, bubble_errors);
        async move { request.await }
    }

    /// Delete all objects in this object store.
    ///
    /// This method returns a future, but always tries to add the object irrespective of whether
    /// the future is ever polled.
    ///
    /// If `bubble_errors = true` then an error here will also cause the transaction to abort,
    /// whether the error is handled or not.
    pub fn clear(
        &self,
        bubble_errors: bool,
    ) -> impl Future<Output = Result<(), errors::ClearError>> {
        let request = self.raw().clear();

        async move {
            Request::new(request.map_err(errors::ClearError::from)?, bubble_errors)
                .await
                .map(|_| ())
                .map_err(errors::ClearError::from)
        }
    }

    /// Delete records from the store that match the given key.
    // TODO delete_range function
    pub async fn delete(
        &self,
        key: impl Into<Key>,
        bubble_errors: bool,
    ) -> Result<(), errors::DeleteError> {
        // give the optimizer the choice of inlining this function or not (minus generics)
        async fn delete_inner(
            this: &ObjectStoreReadWrite,
            key: Key,
            bubble_errors: bool,
        ) -> Result<(), errors::DeleteError> {
            let request = this
                .raw()
                .delete(&key.0)
                .map_err(errors::DeleteError::from)?;
            Request::new(request, bubble_errors)
                .await
                .map(|_| ())
                .map_err(errors::DeleteError::from)
        }

        delete_inner(self, key.into(), bubble_errors).await
    }
}
impl Deref for ObjectStoreReadWrite {
    type Target = ObjectStoreReadOnly;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// An object store during a `readonly` transaction.
///
/// Note that object stores are always sorted with their key values ascending. To iterate in the
/// descending direction use `CursorDirection::Previous`.
#[derive(Debug)]
pub struct ObjectStoreReadOnly {
    inner: IdbObjectStore,
    // TODO cache key path so we can provide better errors when key not supplied?
}

impl ObjectStoreReadOnly {
    pub(crate) fn new(inner: IdbObjectStore) -> Self {
        Self { inner }
    }

    fn raw(&self) -> &IdbObjectStore {
        &self.inner
    }

    /// Get whether this object store uses an auto-incrementing key
    pub fn auto_increment(&self) -> bool {
        self.raw().auto_increment()
    }

    /// Get a list containing all the names of indices on this object store.
    pub fn index_names(&self) -> StringList {
        StringList::new(self.raw().index_names())
    }

    /// Get the key path for the object store.
    // Note: return value should be either null, a DOMString, or a sequence<DOMString> (from w3
    // spec)
    pub fn key_path(&self) -> KeyPath {
        let key_path = self.raw().key_path().unwrap_throw();
        if key_path.is_null() {
            KeyPath::None
        } else if let Some(key_path) = key_path.as_string() {
            KeyPath::String(key_path)
        } else {
            let key_path = key_path.unchecked_into::<js_sys::Iterator>();

            let mut out = vec![];
            for val in &key_path {
                out.push(val.unwrap_throw().as_string().unwrap_throw());
            }
            KeyPath::Sequence(out)
        }
    }

    /// The name of the object store.
    pub fn name(&self) -> String {
        self.raw().name()
    }

    /// Count the number of records in the object store.
    // TODO optional query argument - `count_query` function?
    pub async fn count(&self) -> Result<u32, errors::CountError> {
        let result = Request::new(self.raw().count().map_err(errors::CountError::from)?, false)
            .await
            .map_err(errors::CountError::from)?;
        let result = result.as_f64().expect_throw("unreachable");
        // From reading MDN it seems indexeddb cannot handle counts more than 2^32-1.
        if result <= u32::MAX.into() {
            Ok(result as u32)
        } else {
            throw_str("unreachable")
        }
    }

    /// Get an object from the object store by searching for the given key.
    pub fn get<K, V>(&self, key: K) -> impl Future<Output = Result<Option<V>, errors::GetError>>
    where
        Key: TryFrom<K>,
        V: DeserializeOwned,
    {
        fn get_inner(
            this: &ObjectStoreReadOnly,
            key: Key,
        ) -> impl Future<Output = Result<JsValue, errors::GetError>> {
            let request = this.raw().get(&key.0).map_err(errors::GetError::from);
            async move {
                let request = Request::new(request?, false);
                request.await.map_err(errors::GetError::from)
            }
        }

        let key = match Key::try_from(key) {
            Ok(key) => key,
            Err(_) => return Either::Left(ready(Err(errors::GetError::InvalidKey))),
        };
        Either::Right(get_inner(self, key).map(|output| {
            serde_wasm_bindgen::from_value(output?).map_err(errors::GetError::Deserialize)
        }))
    }

    /// Get an object from the object store by searching for the given key.
    ///
    /// The result will be need to be deserialized from a javascript array, so you should use a
    /// type like `Vec<T>` to deserialize into. The reason we don't return a `Vec` is that you
    /// might want to use a different collection, for example `im::Vector` from the
    /// [`im`](https://crates.io/crates/im) crate, or `futures_signals::SignalVec::MutableVec` from
    /// the [`futures_signals`](https://crates.io/crates/futures_signals) crate
    pub async fn get_all<V>(&self) -> Result<V, errors::GetError>
    where
        V: DeserializeOwned,
    {
        Request::new(self.raw().get_all().map_err(errors::GetError::from)?, false)
            .await
            .map_err(errors::GetError::from)
            .and_then(|val| {
                serde_wasm_bindgen::from_value(val).map_err(errors::GetError::Deserialize)
            })
    }

    /// Open a cursor into the object store.
    ///
    /// This returns a builder - call `build` on it to submit the request. Defaults to iterating
    /// over all values in the store, going forwards.
    ///
    /// # Examples
    ///
    /// > For all examples assume `store` is an open object store
    ///
    /// Iterate over all records in ascending order (like [`get_all`], but doesn't require holding
    /// all records in memory at once)
    ///
    /// ```no_run
    /// use futures::StreamExt;
    ///
    /// let mut iter = store.open_cursor().build();
    /// while let Some(object) = store.next().await {
    ///     // do something with the object
    /// }
    /// ```
    pub fn open_cursor(&self) -> OpenCursor {
        OpenCursor::new(self)
    }
}

/// Builder struct to open a cursor
#[derive(Debug)]
pub struct OpenCursor<'a> {
    store: &'a ObjectStoreReadOnly,
    query: Query,
    direction: CursorDirection,
    bubble_errors: bool,
}

impl<'store> OpenCursor<'store> {
    fn new(store: &'store ObjectStoreReadOnly) -> Self {
        Self {
            store,
            query: Query::ALL,
            direction: CursorDirection::Next,
            bubble_errors: true,
        }
    }

    /// Set the query used to filter the output.
    ///
    /// Records will be returned if the record's key satisfies the query, and skipped otherwise.
    /// The query can be a key, or a range of keys. Note that the conversions into a query can
    /// panic if the range is empty (either the end is before the start, or the end equals the
    /// start and the range doesn't include the upper bound (e.g. for `0..n`))
    pub fn query_key(mut self, query: impl Into<Query>) -> Self {
        self.query = query.into();
        self
    }

    /// Set the direction the cursor should traverse the results in.
    ///
    /// Objects are always stored in ascending order by key, so
    pub fn direction(mut self, direction: CursorDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Execute the request and return an object implementing `Stream<Item = Result<Cursor, _>>`.
    pub fn build(self) -> Result<CursorStream, errors::GetError> {
        let store = self.store.raw();
        let dir = self.direction.into();
        let request = match self.query.inner.as_ref() {
            Some(range) => store.open_cursor_with_range_and_direction(&range, dir),
            None => store.open_cursor_with_range_and_direction(&JsValue::UNDEFINED, dir),
        }
        .map_err(errors::GetError::from)?;
        Ok(CursorStream::new(StreamingRequest::new(
            request,
            self.bubble_errors,
        )))
    }
}
