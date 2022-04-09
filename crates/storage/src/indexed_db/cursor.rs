use super::{errors, StreamingRequest};
use futures::stream::Stream;
use serde::Deserialize;
use std::{
    cell::Cell,
    num::NonZeroU32,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
};
use wasm_bindgen::{prelude::*, throw_str, JsCast, UnwrapThrowExt};
use web_sys::{IdbCursor, IdbCursorDirection, IdbCursorWithValue};

/// Represents an async stream of values from the DB. use the `Stream` impl to access the cursor
/// and its values.
#[derive(Debug)]
pub struct CursorStream {
    /// Every time the request succeeds, its result is an instance of cursor.
    request: StreamingRequest,
}

impl CursorStream {
    pub(crate) fn new(request: StreamingRequest) -> Self {
        Self { request }
    }
}

impl Stream for CursorStream {
    type Item = Result<Cursor, errors::CursorError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.request).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(errors::CursorError::from(e)))),
            Poll::Ready(Some(Ok(next))) => {
                let cursor = next
                    .dyn_into::<IdbCursorWithValue>()
                    .expect_throw("unreachable");
                Poll::Ready(Some(Ok(Cursor::new(cursor))))
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
pub struct Cursor {
    inner: KeyCursor,
}

impl Cursor {
    fn new(inner: IdbCursorWithValue) -> Self {
        Self {
            inner: KeyCursor::new(inner.into()),
        }
    }

    fn raw(&self) -> &IdbCursorWithValue {
        self.inner.inner.unchecked_ref()
    }

    /// Get the value at the current location of this cursor.
    pub fn value_raw(&self) -> JsValue {
        self.raw().value().expect_throw("unreachable")
    }

    /// The value of the object the cursor is currently pointing to.
    pub fn value<V>(&self) -> Result<V, serde_wasm_bindgen::Error>
    where
        V: for<'de> Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(self.value_raw())
    }
}

impl Deref for Cursor {
    type Target = KeyCursor;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Wrapper round IDBCursor
#[derive(Debug)]
pub struct KeyCursor {
    inner: IdbCursor,
    /// Keep track of if the user has advanced the cursor somehow (if they don't we call `advance`
    /// on drop)
    advanced: Cell<bool>,
}

impl KeyCursor {
    fn new(inner: IdbCursor) -> Self {
        Self {
            inner,
            advanced: Cell::new(false),
        }
    }

    /// The direction of the cursor.
    pub fn direction(&self) -> CursorDirection {
        self.inner.direction().into()
    }

    /// Get the primary key for the current record.
    pub fn primary_key_raw(&self) -> JsValue {
        // Unwrap: the `Stream` implementation ensures that the cursor is valid and not moving
        self.inner.primary_key().expect_throw("unreachable")
    }

    /// Get the primary key for the current record.
    pub fn primary_key<K>(&self) -> Result<K, serde_wasm_bindgen::Error>
    where
        K: for<'de> Deserialize<'de>,
    {
        serde_wasm_bindgen::from_value(self.primary_key_raw())
    }

    /// Advance the cursor by the given value.
    pub fn advance(self, amount: NonZeroU32) -> Result<(), errors::AdvanceError> {
        self.inner
            .advance(amount.get())
            .map_err(errors::AdvanceError::from)?;
        self.advanced.set(true);
        Ok(())
    }

    /// Move the cursor on to the next record.
    ///
    /// Equivalent to `cursor.advance(1)`
    pub fn continue_(self) -> Result<(), errors::AdvanceError> {
        self.inner.continue_().map_err(errors::AdvanceError::from)?;
        self.advanced.set(true);
        Ok(())
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
            _ => throw_str("unexpected indexeddb cursor direction"),
        }
    }
}
