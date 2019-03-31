use futures::{sync::oneshot, Async, Future};
use wasm_bindgen::{closure::Closure, JsCast, UnwrapThrowExt};

use crate::blob::BlobLike;

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

    pub fn read_as_string(self, blob: &impl BlobLike) -> ReadAs<String> {
        self.read_as(
            blob,
            |js| js.as_string().unwrap_throw(),
            web_sys::FileReader::read_as_text,
        )
    }

    pub fn read_as_data_url(self, blob: &impl BlobLike) -> ReadAs<String> {
        self.read_as(
            blob,
            |js| js.as_string().unwrap_throw(),
            web_sys::FileReader::read_as_data_url,
        )
    }

    pub fn read_as_array_buffer(self, blob: &impl BlobLike) -> ReadAs<js_sys::ArrayBuffer> {
        self.read_as(
            blob,
            std::convert::Into::into,
            web_sys::FileReader::read_as_array_buffer,
        )
    }

    fn read_as<T, F, G>(self, blob: &impl BlobLike, convert: F, start_read: G) -> ReadAs<T>
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
        ReadAs {
            inner: self.inner,
            receiver: rx,
            _closure: closure,
        }
    }
}

pub struct ReadAs<T> {
    receiver: oneshot::Receiver<T>,
    _closure: Closure<FnMut()>,
    inner: web_sys::FileReader,
}

impl<T> Future for ReadAs<T> {
    type Item = T;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        match self.receiver.poll() {
            Ok(Async::Ready(value)) => Ok(Async::Ready(value)),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(_) => Err(()),
        }
    }
}

// TODO: remove when wasm-bindgen#1387 is fixed
impl<T: std::fmt::Debug> std::fmt::Debug for ReadAs<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("receiver", &self.receiver)
            .field("callback", &"Closure { ... }")
            .finish()
    }
}

impl<T> std::ops::Drop for ReadAs<T> {
    fn drop(&mut self) {
        if self.inner.ready_state() < 2 {
            self.inner.abort();
        }
    }
}
