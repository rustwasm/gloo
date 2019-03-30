use wasm_bindgen::UnwrapThrowExt;

pub trait BlobLike {
    fn size(&self) -> usize {
        self.as_raw().size() as usize
    }

    #[cfg(feature = "mime")]
    fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> {
        self.raw_mime_type().parse()
    }

    fn raw_mime_type(&self) -> String {
        self.as_raw().type_()
    }

    fn as_raw(&self) -> &web_sys::Blob;
}

pub struct Blob {
    inner: web_sys::Blob,
}

impl Blob {
    pub fn new(content: &str) -> Blob {
        let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content));
        let inner = web_sys::Blob::new_with_str_sequence(&parts).unwrap_throw();
        Blob { inner }
    }

    pub fn from_raw(inner: web_sys::Blob) -> Blob {
        Blob { inner }
    }
}

impl BlobLike for Blob {
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }
}
impl BlobLike for File {
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }
}

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
