use web_sys::IdbIndex;

/// An object store index
#[derive(Debug)]
pub struct Index {
    inner: IdbIndex,
}

impl Index {
    pub(crate) fn new(inner: IdbIndex) -> Self {
        Self { inner }
    }
}
