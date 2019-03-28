use futures::sync::oneshot;
use futures::{Async, Future};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::UnwrapThrowExt;

use crate::blob::Blob;

#[derive(Debug)]
pub struct FileReader {
    inner: web_sys::FileReader,
}

pub struct ReadAsString {
    receiver: oneshot::Receiver<String>,
    _closure: Closure<FnMut()>,
}

impl FileReader {
    pub fn new() -> FileReader {
        FileReader {
            inner: web_sys::FileReader::new().unwrap(),
        }
    }

    pub fn read_as_string(self, blob: &impl Blob) -> ReadAsString {
        let (tx, rx) = futures::sync::oneshot::channel();
        let reader = self.inner.clone();
        let closure = Closure::once(move || {
            let _ = reader.result().map(|js_string| {
                let _ = tx.send(js_string.as_string().unwrap_throw());
            });
        });
        let function = closure.as_ref().dyn_ref().unwrap_throw();
        self.inner.clone().set_onload(Some(&function));
        self.inner.read_as_text(&blob.raw()).unwrap_throw();
        ReadAsString {
            receiver: rx,
            _closure: closure,
        }
    }
}

impl Future for ReadAsString {
    type Item = String;
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
impl std::fmt::Debug for ReadAsString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("EventListener")
            .field("receiver", &self.receiver)
            .field("callback", &"Closure { ... }")
            .finish()
    }
}
