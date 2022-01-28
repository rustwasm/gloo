use crate::blob::File;
use wasm_bindgen::prelude::*;

/// A list of files, for example from an `<input type="file">`.
#[derive(Debug, Clone, PartialEq)]
pub struct FileList {
    inner: Vec<File>,
}

impl From<web_sys::FileList> for FileList {
    fn from(raw: web_sys::FileList) -> Self {
        let length = raw.length();

        let inner = (0..length)
            .map(|i| File::from(raw.get(i).unwrap_throw()))
            .collect();

        FileList { inner }
    }
}

impl std::ops::Deref for FileList {
    type Target = [File];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
