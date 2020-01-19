use crate::blob::File;

pub struct FileList {
    inner: Vec<File>,
}

impl std::convert::From<web_sys::FileList> for FileList {
    fn from(js: web_sys::FileList) -> Self {
        let length = js.length();
        let mut inner = Vec::with_capacity(length as usize);
        for i in 0..length {
            inner.push(File::from_raw(js.get(i).unwrap()))
        }
        FileList { inner }
    }
}

impl std::ops::Deref for FileList {
    type Target = [File];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
