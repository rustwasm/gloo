use futures::sync::oneshot;
use futures::{Async, Future};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;

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
        let (tx, rx) = futures::sync::oneshot::channel();
        let reader = self.inner.clone();
        let closure = Closure::once(move || {
            let _ = reader.result().map(|js_string| {
                let _ = tx.send(js_string.as_string().unwrap_throw());
            });
        });
        let function = closure.as_ref().dyn_ref().unwrap_throw();
        self.inner.clone().set_onload(Some(&function));
        self.inner.read_as_text(&blob.as_raw()).unwrap_throw();
        ReadAs {
            inner: self.inner,
            receiver: rx,
            _closure: closure,
        }
    }

    pub fn read_as_data_url(self, blob: &impl BlobLike) -> ReadAs<String> {
        let (tx, rx) = futures::sync::oneshot::channel();
        let reader = self.inner.clone();
        let closure = Closure::once(move || {
            let _ = reader.result().map(|js_string| {
                let _ = tx.send(js_string.as_string().unwrap_throw());
            });
        });
        let function = closure.as_ref().dyn_ref().unwrap_throw();
        self.inner.clone().set_onload(Some(&function));
        self.inner.read_as_data_url(&blob.as_raw()).unwrap_throw();
        ReadAs {
            inner: self.inner,
            receiver: rx,
            _closure: closure,
        }
    }

    pub fn read_as_array_buffer(self, blob: &impl BlobLike) -> ReadAs<js_sys::ArrayBuffer> {
        let (tx, rx) = futures::sync::oneshot::channel();
        let reader = self.inner.clone();
        let closure = Closure::once(move || {
            let _ = reader.result().map(|array_buffer| {
                let _ = tx.send(array_buffer.into());
            });
        });
        let function = closure.as_ref().dyn_ref().unwrap_throw();
        self.inner.clone().set_onload(Some(&function));
        self.inner
            .read_as_array_buffer(&blob.as_raw())
            .unwrap_throw();
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
