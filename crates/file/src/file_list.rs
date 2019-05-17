use crate::blob::File;

pub struct FileList {
    inner: Vec<File>,
}

impl std::convert::Into<FileList> for web_sys::FileList {
    fn into(self) -> FileList {
        let mut inner = Vec::with_capacity(self.length() as usize);
        for i in 0..(self.length()) {
            inner.push(File::from_raw(self.get(i).unwrap()))
        }
        FileList { inner }
    }
}

impl std::ops::Deref for FileList {
    type Target = Vec<File>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
