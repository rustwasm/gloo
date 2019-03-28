pub struct File {
    pub(crate) inner: web_sys::File,
}

impl File {
    pub fn from_raw(inner: web_sys::File) -> File {
        File { inner }
    }

    pub fn as_raw(&self) -> &web_sys::File {
        &self.inner
    }
}
