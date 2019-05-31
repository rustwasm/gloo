pub mod callbacks {
    use super::FileReadError;
    use crate::blob::BlobLike;
    use wasm_bindgen::{closure::Closure, JsCast, UnwrapThrowExt};

    #[derive(Debug)]
    pub struct FileReader {
        inner: web_sys::FileReader,
    }

    impl FileReader {
        pub fn new() -> FileReader {
            FileReader {
                inner: web_sys::FileReader::new().unwrap_throw(),
            }
        }

        pub fn read_to_string<B, F>(self, blob: &B, callback: F)
        where
            B: BlobLike,
            F: FnOnce(Result<String, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_text,
                callback,
            );
        }

        pub fn read_to_data_url<B, F>(self, blob: &B, callback: F)
        where
            B: BlobLike,
            F: FnOnce(Result<String, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_data_url,
                callback,
            );
        }

        pub fn read_to_array_buffer<B, F>(self, blob: &B, callback: F)
        where
            B: BlobLike,
            F: FnOnce(Result<js_sys::ArrayBuffer, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                std::convert::Into::into,
                web_sys::FileReader::read_as_array_buffer,
                callback,
            );
        }

        fn read_as<T, B, ConvertFn, ReadFn, CallbackFn>(
            self,
            blob: &B,
            convert: ConvertFn,
            read: ReadFn,
            callback: CallbackFn,
        ) where
            B: BlobLike,
            ConvertFn: Fn(wasm_bindgen::JsValue) -> T + 'static,
            ReadFn: Fn(&web_sys::FileReader, &web_sys::Blob) -> Result<(), wasm_bindgen::JsValue>,
            CallbackFn: FnOnce(Result<T, FileReadError>) + 'static,
        {
            let reader = self.inner.clone();
            let closure = Closure::once(move || {
                let result = reader
                    .result()
                    .map(convert)
                    .map_err(std::convert::Into::into);
                callback(result);
            });
            let function = closure.as_ref().dyn_ref().unwrap_throw();

            self.inner.clone().set_onload(Some(&function));
            read(&self.inner, blob.as_raw()).unwrap_throw();
        }
    }

    impl std::ops::Drop for FileReader {
        fn drop(&mut self) {
            if self.inner.ready_state() != web_sys::FileReader::DONE {
                self.inner.abort();
            }
        }
    }
}

#[derive(Debug)]
pub enum FileReadError {
    AbortedEarly,
    JsError(wasm_bindgen::JsValue),
}
impl std::convert::From<wasm_bindgen::JsValue> for FileReadError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        FileReadError::JsError(value)
    }
}

impl std::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error reading file")
    }
}

impl std::error::Error for FileReadError {}
