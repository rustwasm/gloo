use once_cell::sync::Lazy;
use wasm_bindgen::{intern, throw_str, UnwrapThrowExt};
use web_sys::DomStringList;

// TODO need to work out what the most efficient way to do this is. Hopefully interning once is the
// best, but it might not make any difference (over interning every time). I don't know the
// internals of `intern`.
static UNREACHABLE_MSG: Lazy<&'static str> = Lazy::new(|| intern("internal error: unreachable"));

pub(crate) trait UnreachableExt<T>: UnwrapThrowExt<T> {
    fn unreachable_throw(self) -> T;
}

impl<T, R: UnwrapThrowExt<T>> UnreachableExt<T> for R {
    fn unreachable_throw(self) -> T {
        self.expect_throw(&UNREACHABLE_MSG)
    }
}

pub(crate) fn unreachable_throw() -> ! {
    throw_str(&UNREACHABLE_MSG)
}

/// A wrapper around [`web_sys::DomStringList`] for easy iteration.
#[derive(Debug)]
pub struct StringList {
    inner: DomStringList,
}

impl StringList {
    pub(crate) fn new(inner: DomStringList) -> Self {
        Self { inner }
    }

    /// The number of strings in this list.
    pub fn len(&self) -> u32 {
        self.inner.length()
    }

    /// Is this list empty?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Does this list contain `value`?
    pub fn contains(&self, value: &str) -> bool {
        self.inner.contains(value)
    }

    /// Get the raw [`DomStringList`][web_sys::DomStringList].
    pub fn into_inner(self) -> DomStringList {
        self.inner
    }
}

impl IntoIterator for StringList {
    type Item = String;
    type IntoIter = StringListIter;
    fn into_iter(self) -> Self::IntoIter {
        StringListIter::new(self.inner)
    }
}

/// An iterator over the `String` contents of `StringList`
#[derive(Debug)]
pub struct StringListIter {
    idx: u32,
    /// Cache length to avoid going through JS quite so many times.
    length: u32,
    inner: DomStringList,
}

impl StringListIter {
    fn new(inner: DomStringList) -> Self {
        Self {
            idx: 0,
            length: inner.length(),
            inner,
        }
    }

    /// Get the original `StringList` back.
    pub fn into_inner(self) -> StringList {
        StringList { inner: self.inner }
    }
}

impl Iterator for StringListIter {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.length {
            return None;
        }
        let out = self.inner.get(self.idx).unwrap_throw();
        self.idx += 1;
        Some(out)
    }
}
