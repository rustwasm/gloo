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

        pub fn read_to_string<F>(self, blob: &impl BlobLike, callback: F)
        where
            F: FnOnce(Result<String, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_text,
                callback,
            );
        }

        pub fn read_to_data_url<F>(self, blob: &impl BlobLike, callback: F)
        where
            F: FnOnce(Result<String, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_data_url,
                callback,
            );
        }

        pub fn read_to_array_buffer<F>(self, blob: &impl BlobLike, callback: F)
        where
            F: FnOnce(Result<js_sys::ArrayBuffer, FileReadError>) + 'static,
        {
            self.read_as(
                blob,
                std::convert::Into::into,
                web_sys::FileReader::read_as_array_buffer,
                callback,
            );
        }

        // fn on_progress<F>(&mut self, callback: F)
        // where
        //     F: FnMut(ProgressEvent) + 'static,
        // {
        //     unimplemented!()
        // }

        // fn on_load_start<F>(&mut self, callback: F)
        // where
        //     F: FnOnce(LoadStartEvent) + 'static,
        // {
        //     unimplemented!()
        // }

        fn read_as<T, ConvertFn, ReadFn, CallbackFn>(
            self,
            blob: &impl BlobLike,
            convert: ConvertFn,
            read: ReadFn,
            callback: CallbackFn,
        ) where
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
            if self.inner.ready_state() < 2 {
                self.inner.abort();
            }
        }
    }

}

pub mod futures {
    use super::FileReadError;
    use crate::blob::BlobLike;
    use futures::{sync::oneshot, Async, Future};
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

        pub fn read_as_string(self, blob: &impl BlobLike) -> ReadAsString {
            let (closure, rx) = self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_text,
            );
            ReadAsString {
                inner: self.inner,
                _closure: closure,
                receiver: rx,
            }
        }

        pub fn read_as_data_url(self, blob: &impl BlobLike) -> ReadAsDataUrl {
            let (closure, rx) = self.read_as(
                blob,
                |js| js.as_string().unwrap_throw(),
                web_sys::FileReader::read_as_data_url,
            );

            ReadAsDataUrl {
                inner: self.inner,
                _closure: closure,
                receiver: rx,
            }
        }

        pub fn read_as_array_buffer(self, blob: &impl BlobLike) -> ReadAsArrayBuffer {
            let (closure, rx) = self.read_as(
                blob,
                std::convert::Into::into,
                web_sys::FileReader::read_as_array_buffer,
            );

            ReadAsArrayBuffer {
                inner: self.inner,
                _closure: closure,
                receiver: rx,
            }
        }

        fn read_as<T, F, G>(
            &self,
            blob: &impl BlobLike,
            convert: F,
            start_read: G,
        ) -> (
            wasm_bindgen::closure::Closure<FnMut() + 'static>,
            futures::sync::oneshot::Receiver<T>,
        )
        where
            T: 'static,
            F: Fn(wasm_bindgen::JsValue) -> T + 'static,
            G: Fn(&web_sys::FileReader, &web_sys::Blob) -> Result<(), wasm_bindgen::JsValue>,
        {
            let (tx, rx) = futures::sync::oneshot::channel();
            let reader = self.inner.clone();
            let closure = Closure::once(move || {
                let _ = reader.result().map(|js_value| {
                    let _ = tx.send(convert(js_value));
                });
            });
            let function = closure.as_ref().dyn_ref().unwrap_throw();
            self.inner.clone().set_onload(Some(&function));
            start_read(&self.inner, blob.as_raw()).unwrap_throw();
            (closure, rx)
        }
    }

    macro_rules! future_impls {
       ($($name:ident : $item:ty),*) => ($(
            pub struct $name {
                receiver: oneshot::Receiver<$item>,
                _closure: Closure<FnMut()>,
                inner: web_sys::FileReader,
            }

            impl Future for $name {
                type Item = $item;
                type Error = FileReadError;

                fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
                    match self.receiver.poll() {
                        Ok(Async::Ready(value)) => Ok(Async::Ready(value)),
                        Ok(Async::NotReady) => Ok(Async::NotReady),
                        Err(e) => Err(e.into()),
                    }
                }
            }

            impl std::ops::Drop for $name {
                fn drop(&mut self) {
                    if self.inner.ready_state() < 2 {
                        self.inner.abort();
                    }
                }
            }
       )*)
    }

    future_impls! { ReadAsString : String, ReadAsDataUrl: String, ReadAsArrayBuffer: js_sys::ArrayBuffer }
}

#[derive(Debug)]
pub enum FileReadError {
    AbortedEarly,
    JsError(wasm_bindgen::JsValue),
}
impl std::convert::Into<FileReadError> for wasm_bindgen::JsValue {
    fn into(self) -> FileReadError {
        FileReadError::JsError(self)
    }
}

impl std::convert::Into<FileReadError> for ::futures::sync::oneshot::Canceled {
    fn into(self) -> FileReadError {
        FileReadError::AbortedEarly
    }
}

impl std::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error reading file")
    }
}

impl std::error::Error for FileReadError {}
