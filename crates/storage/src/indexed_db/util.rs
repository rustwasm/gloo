use wasm_bindgen::UnwrapThrowExt;
use web_sys::DomStringList;

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
