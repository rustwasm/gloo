use super::{errors, util::UnreachableExt, Request, StreamingRequest};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{
    cell::Cell,
    marker::PhantomData,
    ops::Deref,
    pin::Pin,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
    task::{Context, Poll},
};
use wasm_bindgen::{prelude::*, throw_str, JsCast};
use web_sys::{IdbCursor, IdbCursorDirection, IdbCursorWithValue};

#[derive(Debug, Clone)]
struct StreamState {
    /// - `0`: No cursor
    /// - `1`: Active cursor
    /// - `2`: Multi cursors error
    inner: Arc<AtomicU8>,
}

impl StreamState {
    fn new() -> Self {
        Self {
            inner: Arc::new(AtomicU8::new(0)),
        }
    }

    fn take(&self) -> bool {
        if self.inner.load(Ordering::SeqCst) == 0 {
            self.inner
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
        } else {
            // Set to error unconditionally.
            self.inner.store(2, Ordering::SeqCst);
            false
        }
    }

    fn untake(&self) {
        let _ = self
            .inner
            .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst);
    }
}

/// Represents an async stream of values from the DB. use the `Stream` impl to access the cursor
/// and its values.
#[derive(Debug)]
pub struct CursorStream<Ty> {
    /// Every time the request succeeds, its result is an instance of cursor.
    request: StreamingRequest,
    ty: PhantomData<Ty>,
    state: StreamState,
}

impl<Ty> CursorStream<Ty> {
    pub(crate) fn new(request: StreamingRequest) -> Self {
        Self {
            request,
            ty: PhantomData,
            state: StreamState::new(),
        }
    }
}

impl<Ty: Unpin> Stream for CursorStream<Ty> {
    type Item = Result<Cursor<Ty>, errors::LifetimeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.request).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(errors::LifetimeError::from(e)))),
            Poll::Ready(Some(Ok(next))) => {
                let cursor = next.dyn_into::<IdbCursorWithValue>().unwrap_unreachable();
                if self.state.take() {
                    Poll::Ready(Some(Ok(Cursor::new(cursor, self.state.clone()))))
                } else {
                    // TODO add an error type here for overlapping cursors.
                    Poll::Ready(Some(Err(errors::LifetimeError::Unexpected(
                        "overlapping cursors".into(),
                    ))))
                }
            }
        }
    }
}

/// A cursor for iterating through an object store (possibly filtered using a query).
///
/// There are two types of cursors: those with values and those that only have the keys. This is
/// modelled by having `Cursor` (cursors with values) `Deref` to `KeyCursor` (cursors without
/// values).
#[derive(Debug)]
pub struct Cursor<Ty> {
    inner: KeyCursor<Ty>,
}

impl<Ty> Cursor<Ty> {
    fn new(inner: IdbCursorWithValue, state: StreamState) -> Self {
        Self {
            inner: KeyCursor::new(inner.into(), state),
        }
    }

    fn raw(&self) -> &IdbCursorWithValue {
        self.inner.inner.unchecked_ref()
    }

    /// Get the value at the current location of this cursor.
    pub fn value_raw(&self) -> JsValue {
        self.raw().value().unwrap_unreachable()
    }

    /// The value of the object the cursor is currently pointing to.
    pub fn value<V>(&self) -> Result<V, serde_wasm_bindgen::Error>
    where
        V: for<'de> Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(self.value_raw())
    }

    /// Update the value the cursor is currently pointing to.
    ///
    /// Note that the primary key must remain the same. If the primary key is changed (only
    /// possible using in-tree primary keys) then an error will be returned.
    pub async fn update_raw(
        &self,
        updated_value: &JsValue,
        bubble_errors: bool,
    ) -> Result<(), errors::LifetimeError> {
        let req_raw = self.raw().update(updated_value)?;
        Request::new(req_raw, bubble_errors).await?;
        Ok(())
    }

    // TODO we need to handle the error case where the updated value changed the primary key (which
    // will cause an exception). Need a new error type.
    /// Update the value the cursor is currently pointing to.
    ///
    /// Note that the primary key must remain the same. If the primary key is changed (only
    /// possible using in-tree primary keys) then an error will be returned.
    pub async fn update<V>(
        &self,
        updated_value: &V,
        bubble_errors: bool,
    ) -> Result<(), errors::DeSerialize<errors::LifetimeError>>
    where
        V: Serialize,
    {
        let raw_value = serde_wasm_bindgen::to_value(updated_value)?;
        self.update_raw(&raw_value, bubble_errors)
            .await
            .map_err(errors::DeSerialize::Other)
    }

    /// Delete the value the cursor is currently pointing to.
    pub async fn delete(&self, bubble_errors: bool) -> Result<(), errors::LifetimeError> {
        let req_raw = self.raw().delete()?;
        Request::new(req_raw, bubble_errors).await?;
        Ok(())
    }
}

impl<Ty> Deref for Cursor<Ty> {
    type Target = KeyCursor<Ty>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Wrapper round IDBCursor
#[derive(Debug)]
pub struct KeyCursor<Ty> {
    inner: IdbCursor,
    /// Keep track of if the user has advanced the cursor somehow (if they don't we call `advance`
    /// on drop)
    advanced: Cell<bool>,
    /// Ensure that only one cursor object is held at any one time.
    state: StreamState,
    ty: PhantomData<Ty>,
}

impl<Ty> KeyCursor<Ty> {
    fn new(inner: IdbCursor, state: StreamState) -> Self {
        Self {
            inner,
            advanced: Cell::new(false),
            state,
            ty: PhantomData,
        }
    }

    /// The direction of the cursor.
    pub fn direction(&self) -> CursorDirection {
        self.inner.direction().into()
    }

    /// Get the primary key for the current record.
    pub fn primary_key_raw(&self) -> JsValue {
        // Unwrap: the `Stream` implementation ensures that the cursor is valid and not moving
        self.inner.primary_key().unwrap_unreachable()
    }

    /// Get the primary key for the current record.
    pub fn primary_key<K>(&self) -> Result<K, serde_wasm_bindgen::Error>
    where
        K: for<'de> Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(self.primary_key_raw())
    }

    /// Advance the cursor by the given value.
    ///
    /// # Panics
    ///
    /// This function will panic if `amount` is `0`.
    pub fn advance(self, amount: u32) -> Result<(), errors::LifetimeError> {
        if amount == 0 {
            throw_str("advance amount must be > 0");
        }
        self.inner.advance(amount)?;
        self.advanced.set(true);
        Ok(())
    }

    /// Move the cursor on to the next record.
    ///
    /// Equivalent to `cursor.advance(1)`
    pub fn continue_(self) -> Result<(), errors::LifetimeError> {
        self.inner.continue_()?;
        self.advanced.set(true);
        Ok(())
    }
}

impl<Ty> Drop for KeyCursor<Ty> {
    fn drop(&mut self) {
        if !self.advanced.get() {
            // ignore errors
            let _ = self.inner.continue_();
        }
        self.state.untake();
    }
}

/// Possible modes a cursor can be in (fowards or backwards, and unique variants).
///
/// Note that below the term `source` means 'the thing that this cursor points to', which could be
/// a whole object store, or some filtered and/or sorted part of it (e.g. using an index).
// Copy the defn to write our own docs, and panic on unknown constant.
#[derive(Debug)]
pub enum CursorDirection {
    /// This direction causes the cursor to be opened at the start of the source of the cursor.
    ///
    /// When iterated, the cursor should yield all records, including duplicates, in monotonically
    /// increasing order of keys.
    Next,
    /// This direction causes the cursor to be opened at the start of the source of the cursor.
    ///
    /// If multiple records have the same key, then only the first record is included. If
    /// uniqueness of the key is enforced (using a `unique` index constraint) then all keys are
    /// unique and this is the same as `Next`.
    NextUnique,
    /// This direction causes the cursor to be opened at the end of the source of the cursor.
    ///
    /// When iterated, the cursor should yield all records, including duplicates, in monotonically
    /// decreasing order of keys.
    Prev,
    /// This direction causes the cursor to be opened at the end of the source of the cursor.
    ///
    /// If multiple records have the same key, then only the first record is included. If
    /// uniqueness of the key is enforced (using a `unique` index constraint) then all keys are
    /// unique and this is the same as `Next`.
    ///
    /// I'm not sure if 'first' here means the first going forward or going backward. The spec
    /// seems to be ambiguous here. I would guess it means the same as for `NextUnique`.
    PrevUnique,
}

impl From<CursorDirection> for IdbCursorDirection {
    fn from(input: CursorDirection) -> Self {
        match input {
            CursorDirection::Next => IdbCursorDirection::Next,
            CursorDirection::NextUnique => IdbCursorDirection::Nextunique,
            CursorDirection::Prev => IdbCursorDirection::Prev,
            CursorDirection::PrevUnique => IdbCursorDirection::Prevunique,
        }
    }
}

impl From<IdbCursorDirection> for CursorDirection {
    fn from(input: IdbCursorDirection) -> Self {
        match input {
            IdbCursorDirection::Next => CursorDirection::Next,
            IdbCursorDirection::Nextunique => CursorDirection::NextUnique,
            IdbCursorDirection::Prev => CursorDirection::Prev,
            IdbCursorDirection::Prevunique => CursorDirection::PrevUnique,
            _ => throw_str("unreachable"),
        }
    }
}
