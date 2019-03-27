use std::rc::Rc;

use futures::Future;
use wasm_bindgen::JsCast;

use crate::blob::Blob;

pub struct FileReader {
    inner: web_sys::FileReader,
}

impl FileReader {
    pub fn new() -> FileReader {
        FileReader {
            inner: web_sys::FileReader::new().unwrap(),
        }
    }

    pub fn read_as_string(
        self,
        blob: &impl Blob,
    ) -> impl futures::Future<Item = String, Error = ()> {
        let (tx, rx) = futures::sync::oneshot::channel();
        let reader = Rc::new(self.inner);
        let cloned_reader = reader.clone();
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            let _ = cloned_reader.result().map(|r| {
                let _ = tx.send(r.as_string().unwrap());
            });
        });
        let reader = reader.clone();
        let function = cb.dyn_into().unwrap();
        reader.set_onload(Some(&function));
        reader.read_as_text(&blob.raw()).unwrap();
        rx.map_err(|_| ())
    }
}
