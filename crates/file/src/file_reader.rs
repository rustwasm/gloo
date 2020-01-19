pub mod callbacks {
    use super::FileReadError;
    use crate::blob::BlobLike;
    use wasm_bindgen::{closure::Closure, JsCast, UnwrapThrowExt};

    #[derive(Debug)]
    pub struct FileRead {
        reader: web_sys::FileReader,
        closure: Closure<FnMut()>,
    }

    pub fn read_to_string<B, F>(blob: &B, callback: F) -> FileRead
    where
        B: BlobLike,
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read_as(
            blob,
            |js| js.as_string().unwrap_throw(),
            web_sys::FileReader::read_as_text,
            callback,
        )
    }

    pub fn read_to_data_url<B, F>(blob: &B, callback: F) -> FileRead
    where
        B: BlobLike,
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read_as(
            blob,
            |js| js.as_string().unwrap_throw(),
            web_sys::FileReader::read_as_data_url,
            callback,
        )
    }

    pub fn read_to_array_buffer<B, F>(blob: &B, callback: F) -> FileRead
    where
        B: BlobLike,
        F: FnOnce(Result<js_sys::ArrayBuffer, FileReadError>) + 'static,
    {
        read_as(
            blob,
            std::convert::Into::into,
            web_sys::FileReader::read_as_array_buffer,
            callback,
        )
    }

    fn read_as<T, B, ConvertFn, ReadFn, CallbackFn>(
        blob: &B,
        convert: ConvertFn,
        read: ReadFn,
        callback: CallbackFn,
    ) -> FileRead
    where
        B: BlobLike,
        ConvertFn: Fn(wasm_bindgen::JsValue) -> T + 'static,
        ReadFn: Fn(&web_sys::FileReader, &web_sys::Blob) -> Result<(), wasm_bindgen::JsValue>,
        CallbackFn: FnOnce(Result<T, FileReadError>) + 'static,
    {
        let reader = web_sys::FileReader::new().unwrap_throw();
        let clone = reader.clone();
        let closure = Closure::once(move || {
            let result = clone
                .result()
                .map(convert)
                .map_err(std::convert::Into::into);
            callback(result);
        });
        let function = closure.as_ref().dyn_ref().unwrap_throw();

        reader.clone().set_onload(Some(&function));
        read(&reader, blob.as_raw()).unwrap_throw();
        FileRead { reader, closure }
    }

    impl std::ops::Drop for FileRead {
        fn drop(&mut self) {
            if self.reader.ready_state() != web_sys::FileReader::DONE {
                self.reader.abort();
            }
        }
    }
}

#[derive(Debug)]
pub enum FileReadError {
    AbortedEarly,
    JsError(js_sys::Error),
}
impl std::convert::From<wasm_bindgen::JsValue> for FileReadError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        FileReadError::JsError(value.into())
    }
}

impl std::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            FileReadError::AbortedEarly => String::from("aborted early"),
            FileReadError::JsError(error) => error.message().into(),
        };

        write!(f, "FileReader error: {}", msg)
    }
}

impl std::error::Error for FileReadError {}
