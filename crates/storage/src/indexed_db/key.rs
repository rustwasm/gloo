use super::{errors, indexed_db};
use std::{
    cmp::Ordering,
    convert::TryFrom,
    ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};
use wasm_bindgen::{prelude::*, throw_str};
use web_sys::IdbKeyRange;

mod sealed {
    pub trait Sealed {}

    impl<'a> Sealed for &'a str {}
    impl<'a, T> Sealed for &'a [T] where T: AsRef<str> + 'a {}
    impl Sealed for super::KeyPath {}
}

// Key path
// --------

/// A trait for types that can be used as a key path when creating an object store.
///
/// Types allowed are either a string or an array of strings. An empty string is
/// equivalent to not setting the key path.
pub trait IntoKeyPath: sealed::Sealed {
    /// Internal - please ignore
    ///
    /// Converts self into a value to use as the keyPath (must be a JsValue)
    fn into_jsvalue(self) -> JsValue;
}

impl<'a> IntoKeyPath for &'a str {
    fn into_jsvalue(self) -> JsValue {
        JsValue::from(self)
    }
}

impl<'a, T> IntoKeyPath for &'a [T]
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

impl IntoKeyPath for KeyPath {
    fn into_jsvalue(self) -> JsValue {
        match self {
            KeyPath::None => JsValue::NULL,
            KeyPath::String(s) => JsValue::from(s),
            KeyPath::Sequence(multi) => multi
                .iter()
                .map(|s| JsValue::from(s))
                .collect::<js_sys::Array>()
                .into(),
        }
    }
}

/// The different types that are allowed to be a key path.
#[derive(Debug)]
pub enum KeyPath {
    /// No key path
    None,
    /// Single key path
    String(String),
    /// Multiple key paths
    Sequence(Vec<String>),
}

// Key
// ---

/// A valid indexedDB key
///
/// # From [the spec]
///
/// The following ECMAScript types are valid keys:
///
/// - Number primitive values, except NaN. This includes Infinity and -Infinity.
/// - Date objects, except where the DateValue internal slot is NaN.
/// - String primitive values.
/// - ArrayBuffer objects (or views on buffers such as Uint8Array).
/// - Array objects, where every item is defined, is itself a valid key, and does not directly or
///   indirectly contain itself. This includes empty arrays. Arrays can contain other arrays.
///
/// Attempting to convert other ECMAScript values to a key will fail.
///
/// # Extra notes
///
/// Keys are compared (for Eq, Ord, etc) using `window.indexedDB.cmp`. If indexeddb is not
/// supported, or the values are not valid keys, then the comparison functions will panic
///
/// [the spec]: https://w3c.github.io/IndexedDB/#key-construct
#[derive(Debug)]
pub struct Key(pub(crate) JsValue);

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Key {}

impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Ord::cmp(self, other))
    }
}

// My argument is that if these keys are used in indexes to a database, they have to obey the
// ordering rules for B-trees, therefore they almost certainly fulfil the contract for Eq/Ord.
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        match indexed_db()
            .expect_throw("indexeddb not supported")
            .cmp(&self.0, &other.0)
            .expect_throw("invalid key in indexedDB.cmp")
        {
            -1 => Ordering::Less,
            0 => Ordering::Equal,
            1 => Ordering::Greater,
            _ => throw_str("unreachable"),
        }
    }
}

impl TryFrom<f64> for Key {
    type Error = errors::NumberIsNan;

    fn try_from(input: f64) -> Result<Self, Self::Error> {
        if input.is_nan() {
            Err(errors::NumberIsNan)
        } else {
            Ok(Key(JsValue::from_f64(input)))
        }
    }
}

impl TryFrom<&js_sys::Date> for Key {
    type Error = errors::NumberIsNan;

    fn try_from(input: &js_sys::Date) -> Result<Self, Self::Error> {
        if input.value_of().is_nan() {
            Err(errors::NumberIsNan)
        } else {
            Ok(Key(JsValue::from(input)))
        }
    }
}

impl From<&str> for Key {
    fn from(input: &str) -> Self {
        Key(JsValue::from_str(input))
    }
}

impl From<&js_sys::ArrayBuffer> for Key {
    fn from(input: &js_sys::ArrayBuffer) -> Self {
        Key(JsValue::from(input))
    }
}

// TODO From<ArrayBuffer views>
// TODO figure out how to implement for &[T] where Key: (Try)From<T> - we should be able to build
// the array on the fly and save the user having to build the `[Key]`.

impl From<&[Key]> for Key {
    fn from(input: &[Key]) -> Self {
        let array = js_sys::Array::new();
        for el in input.iter() {
            array.push(&el.0);
        }
        Key(array.into())
    }
}

/// A query to filter a sequence of records (to those that match the query).
///
/// It is either no restriction (`Query::ALL`), a specific value of the `Key`, or a range of
/// `Key` values.
#[derive(Debug)]
pub struct Query {
    /// `None` means `all records`
    pub(crate) inner: Option<IdbKeyRange>,
}

impl Query {
    /// A special range that includes all records in a store/index.
    pub const ALL: Self = Self { inner: None };

    /// Create a new `Query`.
    fn new(inner: Result<IdbKeyRange, JsValue>) -> Self {
        Self {
            inner: Some(inner.expect_throw("keyrange error not caught (should be unreachable)")),
        }
    }
}

impl From<Range<Key>> for Query {
    fn from(range: Range<Key>) -> Self {
        if range.start >= range.end {
            throw_str("lower bound was >= upper bound (the range is empty)");
        }
        Self::new(IdbKeyRange::bound_with_lower_open_and_upper_open(
            &range.start.0,
            &range.end.0,
            false,
            true,
        ))
    }
}

impl From<RangeInclusive<Key>> for Query {
    fn from(range: RangeInclusive<Key>) -> Self {
        if range.start() > range.end() {
            throw_str("lower bound was > upper bound (the range is empty)");
        }
        Self::new(IdbKeyRange::bound(&range.start().0, &range.end().0))
    }
}

impl From<RangeFrom<Key>> for Query {
    fn from(range: RangeFrom<Key>) -> Self {
        Self::new(IdbKeyRange::lower_bound(&range.start.0))
    }
}

impl From<RangeTo<Key>> for Query {
    fn from(range: RangeTo<Key>) -> Self {
        Self::new(IdbKeyRange::upper_bound_with_open(&range.end.0, true))
    }
}

impl From<RangeToInclusive<Key>> for Query {
    fn from(range: RangeToInclusive<Key>) -> Self {
        Self::new(IdbKeyRange::upper_bound(&range.end.0))
    }
}

impl From<Key> for Query {
    fn from(key: Key) -> Self {
        Self::new(IdbKeyRange::only(&key.0))
    }
}

impl From<RangeFull> for Query {
    fn from(_: RangeFull) -> Self {
        Self::ALL
    }
}
