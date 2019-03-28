pub use crate::file::File;

use wasm_bindgen::UnwrapThrowExt;

pub trait Blob: raw::Blob {
    fn size(&self) -> usize {
        self.raw().size() as usize
    }

    fn mime_type(&self) -> Option<mime::Mime> {
        Some(self.raw().type_().parse().ok()?)
    }
}

pub struct DataBlob {
    inner: web_sys::Blob,
}

impl DataBlob {
    pub fn new(content: &str) -> DataBlob {
        let parts = js_sys::Array::of1(&wasm_bindgen::JsValue::from_str(content));
        let inner = web_sys::Blob::new_with_str_sequence(&parts).unwrap_throw();
        DataBlob { inner }
    }
}

impl Blob for DataBlob {}
impl Blob for File {}

mod raw {
    pub trait Blob {
        fn raw(&self) -> &web_sys::Blob;
    }

    impl Blob for super::File {
        fn raw(&self) -> &web_sys::Blob {
            &self.inner
        }
    }

    impl Blob for super::DataBlob {
        fn raw(&self) -> &web_sys::Blob {
            &self.inner
        }
    }
}
