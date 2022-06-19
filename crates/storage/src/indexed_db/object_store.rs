// NOTE: all transaction operations must be started on the *same tick* (i.e. not in an async block)
// otherwise with transaction will auto-commit before the operation is started.
use super::{
    errors,
    util::{unreachable_throw, UnreachableExt},
    CursorDirection, CursorStream, Index, IntoKeyPath, Key, KeyPath, Query, ReadOnly, ReadWrite,
    Request, StreamingRequest, StringList, Upgrade,
};
use serde::{Deserialize, Serialize};
use std::{future::Future, marker::PhantomData};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{IdbIndexParameters, IdbObjectStore};

/// An indexedDB object store.
///
/// The type `Ty` denotes what context the store exists (`Upgrade`, `ReadWrite`, or `ReadOnly`).
#[derive(Debug)]
pub struct ObjectStore<Ty> {
    inner: IdbObjectStore,
    ty: PhantomData<Ty>,
}

impl<Ty> ObjectStore<Ty> {
    /// Contract: the caller is responsible for choosing the right subtype
    pub(crate) fn new(inner: IdbObjectStore) -> Self {
        Self {
            inner,
            ty: PhantomData,
        }
    }

    fn raw(&self) -> &IdbObjectStore {
        &self.inner
    }

    // We actually implement all methods privately for all variants and then publicly expose only
    // those methods that are valid. Hopefully this should help reduce code duplication (TODO does
    // it actually?)

    // Only valid during upgrade

    /// Changes the object store's name.
    ///
    /// # Panics
    ///
    /// Currently this method will panic on error. If/when [this wasm-bindgen patch](https://github.com/rustwasm/wasm-bindgen/pull/2852)
    /// lands errors will be returned instead. Because of the return type, the change will be
    /// backwards-compatible.
    fn set_name_inner(&self, new_name: &str) -> Result<(), errors::SetNameError> {
        self.raw().set_name(new_name);
        Ok(())
    }

    /// Create a new index for the object store.
    fn create_index_inner<ITy>(
        &self,
        name: &str,
        opts: IndexOptions,
    ) -> Result<Index<ITy>, errors::CreateIndexError> {
        self.inner
            .create_index_with_str_sequence_and_optional_parameters(
                name,
                &opts.key_path,
                &opts.params,
            )
            .map(Index::new)
            .map_err(errors::CreateIndexError::from)
    }

    /// Delete the index with the given name.
    fn delete_index_inner(&self, name: &str) -> Result<(), errors::DeleteIndexError> {
        self.raw()
            .delete_index(name)
            .map_err(errors::DeleteIndexError::from)
    }

    // Valid during upgrade or read/write transaction

    /// Add an object to the database.
    ///
    /// This method returns a future, but always tries to add the object irrespective of whether
    /// the future is ever polled. If `bubble_errors = true` any errors returned here will also
    /// cause the transaction to abort.
    async fn add_raw_inner(
        &self,
        value: &JsValue,
        key: Option<Key>,
        bubble_errors: bool,
    ) -> Result<(), errors::AddError> {
        let request = if let Some(key) = key {
            self.raw().add_with_key(value, &key)
        } else {
            self.raw().add(value)
        };

        let request = match request {
            Ok(request) => request,
            Err(e) => return Err(errors::AddError::from(e)),
        };
        match Request::new(request, bubble_errors).await {
            Ok(_) => Ok(()),
            Err(e) => Err(errors::AddError::from(e)),
        }
    }

    /// Add an arbitrary object to the database using serde to serialize it to a JsValue.
    async fn add_inner(
        &self,
        value: &(impl Serialize + ?Sized),
        key: Option<Key>,
        bubble_errors: bool,
    ) -> Result<(), errors::AddError> {
        // TODO handle errors
        let value = serde_wasm_bindgen::to_value(value).unreachable_throw();
        self.add_raw_inner(&value, key, bubble_errors).await
    }

    async fn put_raw_inner(
        &self,
        value: &JsValue,
        key: Option<Key>,
        bubble_errors: bool,
    ) -> Result<(), errors::AddError> {
        let request = if let Some(key) = key {
            self.raw().put_with_key(value, &key)
        } else {
            self.raw().put(value)
        };

        let request = match request {
            Ok(request) => request,
            Err(e) => return Err(errors::AddError::from(e)),
        };
        match Request::new(request, bubble_errors).await {
            Ok(_) => Ok(()),
            Err(e) => Err(errors::AddError::from(e)),
        }
    }

    async fn put_inner(
        &self,
        value: &(impl Serialize + ?Sized),
        key: Option<Key>,
        bubble_errors: bool,
    ) -> Result<(), errors::AddError> {
        // TODO handle errors
        let value = serde_wasm_bindgen::to_value(value).unreachable_throw();
        self.put_raw_inner(&value, key, bubble_errors).await
    }

    /// Delete all objects in this object store.
    ///
    /// This method returns a future, but always tries to add the object irrespective of whether
    /// the future is ever polled.
    ///
    /// If `bubble_errors = true` then an error here will also cause the transaction to abort,
    /// whether the error is handled or not.
    fn clear_inner(
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
    fn delete_inner(
        &self,
        key: Key,
        bubble_errors: bool,
    ) -> impl Future<Output = Result<(), errors::DeleteError>> {
        let request = self.inner.delete(&key);
        async move {
            let request = request.map_err(errors::DeleteError::from)?;
            Request::new(request, bubble_errors)
                .await
                .map(|_| ())
                .map_err(errors::DeleteError::from)
        }
    }

    /// Execute the request and return an object implementing `Stream<Item = Result<Cursor, _>>`.
    fn cursor_inner<CurTy>(
        &self,
        opts: CursorOptions,
    ) -> Result<CursorStream<CurTy>, errors::LifetimeError> {
        let dir = opts.direction.into();
        let query = opts.query.as_ref();
        let request = self
            .inner
            .open_cursor_with_range_and_direction(query, dir)?;
        Ok(CursorStream::new(StreamingRequest::new(
            request,
            opts.bubble_errors,
        )))
    }

    // Always valid (read only)

    /// Whether this object store uses an auto-incrementing key
    pub fn is_auto_increment(&self) -> bool {
        self.inner.auto_increment()
    }

    /// Get a list containing all the names of indices on this object store.
    pub fn index_names(&self) -> StringList {
        StringList::new(self.inner.index_names())
    }

    /// Get the key path for the object store.
    // Note: return value should be either null, a DOMString, or a sequence<DOMString> (from w3
    // spec)
    pub fn key_path_inner(&self) -> KeyPath {
        let key_path = self.inner.key_path().unreachable_throw();
        if key_path.is_null() {
            KeyPath::None
        } else if let Some(key_path) = key_path.as_string() {
            KeyPath::String(key_path)
        } else {
            let key_path = key_path.unchecked_into::<js_sys::Iterator>();

            let mut out = vec![];
            for val in &key_path {
                out.push(val.unreachable_throw().as_string().unreachable_throw());
            }
            KeyPath::Sequence(out)
        }
    }

    /// The name of the object store.
    pub fn name(&self) -> String {
        self.inner.name()
    }

    /// Count the number of records in the object store.
    ///
    /// To count all objects pass `&Query::ALL`.
    pub async fn count(&self, query: &Query) -> Result<u32, errors::CountError> {
        let result = Request::new(
            self.inner
                .count_with_key(query.as_ref())
                .map_err(errors::CountError::from)?,
            false,
        )
        .await
        .map_err(errors::CountError::from)?;
        let result = result.as_f64().unreachable_throw();
        // From reading MDN it seems indexeddb cannot handle counts more than 2^32-1.
        if result <= u32::MAX.into() {
            Ok(result as u32)
        } else {
            unreachable_throw()
        }
    }

    /// Get an object from the object store by searching for the given key.
    pub async fn get_raw(&self, key: Key) -> Result<JsValue, errors::LifetimeError> {
        let request = self.inner.get(&key)?;
        Ok(Request::new(request, false).await?)
    }

    /// Get an object from the object store by searching for the given key.
    ///
    /// Automatically deserializes the result.
    pub async fn get<V>(
        &self,
        key: Key,
    ) -> Result<Option<V>, errors::DeSerialize<errors::LifetimeError>>
    where
        V: for<'de> Deserialize<'de>,
    {
        Ok(serde_wasm_bindgen::from_value(
            self.get_raw(key)
                .await
                .map_err(errors::DeSerialize::Other)?,
        )?)
    }

    /// Get all objects from the object store matching the given query.
    ///
    /// Use `Query::ALL` to get all values.
    ///
    /// The second argument is the maximum number of objects to return. If `None`, all matching
    /// objects will be returned.
    pub async fn get_all_raw(
        &self,
        query: &Query,
        limit: Option<u32>,
    ) -> Result<JsValue, errors::LifetimeError> {
        let request = match limit {
            Some(limit) => self
                .inner
                .get_all_with_key_and_limit(query.as_ref(), limit)?,
            None => self.inner.get_all_with_key(query.as_ref())?,
        };
        Ok(Request::new(request, false).await?)
    }

    /// Get a sequence of values
    ///
    /// The user should choose a collection type `C` that can deserialize a sequence of values (for
    /// example `Vec` from the standard library).
    ///
    /// Use `Query::ALL` to get all values.
    ///
    /// The second argument is the maximum number of objects to return. If `None`, all matching
    /// objects will be returned.
    pub async fn get_all<C>(
        &self,
        query: &Query,
        limit: Option<u32>,
    ) -> Result<C, errors::DeSerialize<errors::LifetimeError>>
    where
        C: for<'de> Deserialize<'de>,
    {
        Ok(serde_wasm_bindgen::from_value(
            self.get_all_raw(query, limit)
                .await
                .map_err(errors::DeSerialize::Other)?,
        )?)
    }

    /// Open an index with the given name.
    ///
    /// Returns `None` if no index with the given name exists.
    pub fn index(&self, name: &str) -> Result<Option<Index<Ty>>, errors::LifetimeError> {
        match self.inner.index(name) {
            Ok(idx) => Ok(Some(Index::new(idx))),
            Err(e) => {
                let e = errors::LifetimeError::from(e);
                if matches!(
                    &e,
                    errors::LifetimeError::Unexpected(msg) if msg.as_str() == "NotFoundError")
                {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }
}

impl ObjectStore<Upgrade> {
    /// Changes the object store's name.
    ///
    /// # Panics
    ///
    /// Currently this method will panic on error. If/when [this wasm-bindgen patch](https://github.com/rustwasm/wasm-bindgen/pull/2852)
    /// lands errors will be returned instead. Because of the return type, the change will be
    /// backwards-compatible.
    pub fn set_name(&self, new_name: &str) -> Result<(), errors::SetNameError> {
        self.set_name_inner(new_name)
    }

    /// Create a new index for the object store.
    pub fn create_index(
        &self,
        name: &str,
        opts: IndexOptions,
    ) -> Result<Index<Upgrade>, errors::CreateIndexError> {
        self.create_index_inner(name, opts)
    }

    /// Delete the index with the given name.
    pub fn delete_index(&self, name: &str) -> Result<(), errors::DeleteIndexError> {
        self.delete_index_inner(name)
    }
}

macro_rules! impl_ReadWrite {
    ($ty:ty) => {
        impl $ty {
            /// Add an object to the database.
            ///
            /// This method returns a future, but always tries to add the object irrespective of whether
            /// the future is ever polled. If `bubble_errors = true` any errors returned here will also
            /// cause the transaction to abort.
            pub async fn add_raw(
                &self,
                value: &JsValue,
                key: Option<Key>,
                bubble_errors: bool,
            ) -> Result<(), errors::AddError> {
                self.add_raw_inner(value, key, bubble_errors).await
            }

            /// Add an arbitrary object to the database using serde to serialize it to a JsValue.
            pub async fn add(
                &self,
                value: &(impl Serialize + ?Sized),
                key: Option<Key>,
                bubble_errors: bool,
            ) -> Result<(), errors::AddError> {
                self.add_inner(value, key, bubble_errors).await
            }

            /// Update an object in the database using serde to serialize it to a JsValue.
            pub async fn put_raw(
                &self,
                value: &JsValue,
                key: Option<Key>,
                bubble_errors: bool,
            ) -> Result<(), errors::AddError> {
                self.put_raw_inner(value, key, bubble_errors).await
            }

            /// Update an object in the database using serde to serialize it to a JsValue.
            pub async fn put(
                &self,
                value: &(impl Serialize + ?Sized),
                key: Option<Key>,
                bubble_errors: bool,
            ) -> Result<(), errors::AddError> {
                self.put_inner(value, key, bubble_errors).await
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
                self.clear_inner(bubble_errors)
            }

            /// Delete records from the store that match the given key.
            pub fn delete(
                &self,
                key: impl Into<Key>,
                bubble_errors: bool,
            ) -> impl Future<Output = Result<(), errors::DeleteError>> {
                self.delete_inner(key.into(), bubble_errors)
            }

            /// Iterate over records in the store using a cursor.
            pub fn cursor(
                &self,
                opts: CursorOptions,
            ) -> Result<CursorStream<ReadWrite>, errors::LifetimeError> {
                self.cursor_inner(opts)
            }
        }
    };
}

impl_ReadWrite!(ObjectStore<Upgrade>);
impl_ReadWrite!(ObjectStore<ReadWrite>);

impl ObjectStore<ReadOnly> {
    /// Iterate over records in the store using a cursor.
    pub fn cursor(
        &self,
        opts: CursorOptions,
    ) -> Result<CursorStream<ReadOnly>, errors::LifetimeError> {
        self.cursor_inner(opts)
    }
}

/// A builder object for creating an index.
///
/// Once you have set the options you require, call `build` to run the operation.
#[derive(Debug)]
pub struct IndexOptions {
    key_path: JsValue,
    params: IdbIndexParameters,
}

impl Default for IndexOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexOptions {
    /// The default options
    pub fn new() -> Self {
        Self {
            key_path: JsValue::UNDEFINED,
            params: IdbIndexParameters::new(),
        }
    }

    /// Set the key path for the index.
    pub fn key_path(mut self, key_path: impl IntoKeyPath) -> Self {
        self.key_path = key_path.into_jsvalue();
        self
    }

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
}

/// Options when opening a cursor.
#[derive(Debug)]
pub struct CursorOptions {
    query: Query,
    direction: CursorDirection,
    bubble_errors: bool,
}

impl CursorOptions {
    fn new() -> Self {
        Self {
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

    /// Whether errors should abort the whole transaction.
    pub fn bubble_errors(mut self, bubble_errors: bool) -> Self {
        self.bubble_errors = bubble_errors;
        self
    }
}

impl Default for CursorOptions {
    fn default() -> Self {
        Self::new()
    }
}
