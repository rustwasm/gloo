use wasm_bindgen::throw_str;

pub mod callbacks {
    use crate::{
        blob::Blob,
        file_reader::{FileReadError, ReadyState},
    };
    use gloo_events::EventListener;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{prelude::*, throw_str, JsCast, UnwrapThrowExt};

    /// A guard object that aborts the file read when dropped (if the read isn't already
    /// finished).
    #[derive(Debug)]
    pub struct FileReader {
        reader: web_sys::FileReader,
        _load_listener: EventListener,
        _error_listener: EventListener,
    }

    impl std::ops::Drop for FileReader {
        fn drop(&mut self) {
            if !ReadyState::from(self.reader.ready_state()).is_done() {
                self.reader.abort();
            }
        }
    }

    /// Asynchronously converts `blob` into a text string and then passes it to the `callback`.
    ///
    /// If the returned `FileReader` is dropped before the callback is called, the read will be
    /// cancelled.
    pub fn read_as_text<F>(blob: &Blob, callback: F) -> FileReader
    where
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read(
            blob,
            callback,
            |value| value.as_string().unwrap_throw(),
            |reader, blob| reader.read_as_text(blob).unwrap_throw(),
        )
    }

    /// Asynchronously converts the `blob` into a base64 encoded `data:` URL and then passes it to
    /// the `callback`.
    ///
    /// If the returned `FileReader` is dropped before the callback is called, the read will be
    /// cancelled.
    pub fn read_as_data_url<F>(blob: &Blob, callback: F) -> FileReader
    where
        F: FnOnce(Result<String, FileReadError>) + 'static,
    {
        read(
            blob,
            callback,
            |value| value.as_string().unwrap_throw(),
            |reader, blob| reader.read_as_data_url(blob).unwrap_throw(),
        )
    }

    /// Asynchronously converts the `blob` into an array buffer and then passes it to the `callback`.
    ///
    /// If the returned `FileReader` is dropped before the callback is called, the read will be
    /// cancelled.
    pub fn read_as_array_buffer<F>(blob: &Blob, callback: F) -> FileReader
    where
        F: FnOnce(Result<js_sys::ArrayBuffer, FileReadError>) + 'static,
    {
        read(
            blob,
            callback,
            |value| value.dyn_into::<js_sys::ArrayBuffer>().unwrap_throw(),
            |reader, blob| reader.read_as_array_buffer(blob).unwrap_throw(),
        )
    }

    /// Asynchronously converts the `blob` into a `Vec<u8>` and then passes it to the `callback`.
    ///
    /// If the returned `FileReader` is dropped before the callback is called, the read will be
    /// cancelled.
    pub fn read_as_bytes<F>(blob: &Blob, callback: F) -> FileReader
    where
        F: FnOnce(Result<Vec<u8>, FileReadError>) + 'static,
    {
        read_as_array_buffer(blob, move |result| {
            callback(result.map(|buffer| js_sys::Uint8Array::new(&buffer).to_vec()));
        })
    }

    /// Generic function to start the async read of the `FileReader`.
    ///
    /// `callback` is the user-supplied callback, `extract_fn` function converts between JsValue
    /// returned by the read and the type the callback expects, and `read_fn` is the method that
    /// runs the async read on the `FileReader`.
    fn read<T, CF, EF, RF>(blob: &Blob, callback: CF, extract_fn: EF, read_fn: RF) -> FileReader
    where
        CF: FnOnce(Result<T, FileReadError>) + 'static,
        EF: Fn(JsValue) -> T + 'static,
        RF: Fn(&web_sys::FileReader, &web_sys::Blob) + 'static,
    {
        // we need to be able to run the FnOnce, while proving to the compiler that it can only run
        // once. The easiest way is to `take` it out of an Option. The `Rc` and `RefCell` are
        // because we need shared ownership and mutability (for the `take`).
        let load_callback: Rc<RefCell<Option<CF>>> = Rc::new(RefCell::new(Some(callback)));
        let error_callback = load_callback.clone();

        let reader = web_sys::FileReader::new().unwrap_throw();

        let load_reader = reader.clone();
        let error_reader = reader.clone();
        // Either the load listener or the error listener will be called, never both (so FnOnce is
        // ok).
        let load_listener = EventListener::new(&reader, "load", move |_event| {
            let result = extract_fn(load_reader.result().unwrap_throw());
            let callback = load_callback.borrow_mut().take().unwrap_throw();
            callback(Ok(result));
        });

        let error_listener = EventListener::new(&reader, "error", move |_event| {
            let exception = error_reader.error().unwrap_throw();
            let error = match exception.name().as_str() {
                "NotFoundError" => FileReadError::NotFound(exception.message()),
                "NotReadableError" => FileReadError::NotReadable(exception.message()),
                "SecurityError" => FileReadError::Security(exception.message()),
                // This branch should never be hit, so returning a less helpful error message is
                // less of an issue than pulling in `format!` code.
                _ => throw_str("unrecognised error type"),
            };
            let callback = error_callback.borrow_mut().take().unwrap_throw();
            callback(Err(error));
        });

        read_fn(&reader, blob.as_ref());

        FileReader {
            reader,
            _load_listener: load_listener,
            _error_listener: error_listener,
        }
    }
}

#[cfg(feature = "futures")]
pub mod futures {
    use crate::{Blob, FileReadError};
    use std::future::Future;
    use wasm_bindgen::UnwrapThrowExt;

    /// Returns the contents of `blob` as a text string.
    ///
    /// Equivalent to `async fn read_as_text(blob: &Blob) -> Result<String, FileReadError>` but
    /// without borrowing the `Blob` fore the lifetime of the future.
    pub fn read_as_text(blob: &Blob) -> impl Future<Output = Result<String, FileReadError>> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let reader = super::callbacks::read_as_text(blob, |result| {
            sender.send(result).unwrap_throw();
        });

        async move {
            let output = receiver.await.unwrap_throw();
            drop(reader);
            output
        }
    }

    /// Returns the contents of `blob` as a base64 encoded `data:` URL.
    ///
    /// Equivalent to `async fn read_as_data_url(blob: &Blob) -> Result<String, FileReadError>` but
    /// without borrowing the `Blob` fore the lifetime of the future.
    pub fn read_as_data_url(blob: &Blob) -> impl Future<Output = Result<String, FileReadError>> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let reader = super::callbacks::read_as_data_url(blob, |result| {
            sender.send(result).unwrap_throw();
        });

        async move {
            let output = receiver.await.unwrap_throw();
            drop(reader);
            output
        }
    }

    /// Returns the contents of `blob` as an array buffer.
    ///
    /// Equivalent to
    /// `async fn read_as_array_buffer(blob: &Blob) -> Result<js_sys::ArrayBuffer, FileReadError>`
    /// but without borrowing the `Blob` fore the lifetime of the future.
    pub fn read_as_array_buffer(
        blob: &Blob,
    ) -> impl Future<Output = Result<js_sys::ArrayBuffer, FileReadError>> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let reader = super::callbacks::read_as_array_buffer(blob, |result| {
            sender.send(result).unwrap_throw();
        });

        async move {
            let output = receiver.await.unwrap_throw();
            drop(reader);
            output
        }
    }

    /// Returns the contents of `blob` as a `Vec<u8>`.
    ///
    /// Equivalent to
    /// `async fn read_as_bytes(blob: &Blob) -> Result<Vec<u8>, FileReadError>`
    /// but without borrowing the `Blob` fore the lifetime of the future.
    pub fn read_as_bytes(blob: &Blob) -> impl Future<Output = Result<Vec<u8>, FileReadError>> {
        let (sender, receiver) = futures_channel::oneshot::channel();
        let reader = super::callbacks::read_as_bytes(blob, |result| {
            sender.send(result).unwrap_throw();
        });

        async move {
            let output = receiver.await.unwrap_throw();
            drop(reader);
            output
        }
    }
}

enum ReadyState {
    Empty,
    Loading,
    Done,
}

impl ReadyState {
    fn is_done(&self) -> bool {
        matches!(self, ReadyState::Done)
    }
}

impl From<u16> for ReadyState {
    fn from(val: u16) -> Self {
        match val {
            0 => ReadyState::Empty,
            1 => ReadyState::Loading,
            2 => ReadyState::Done,
            _ => throw_str("got invalid value for FileReader.readyState"),
        }
    }
}

#[derive(Debug)]
pub enum FileReadError {
    AbortedEarly,
    NotFound(String),
    NotReadable(String),
    Security(String),
}

impl std::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FileReadError::AbortedEarly => write!(f, "FileReader aborted early"),
            FileReadError::NotFound(msg) => write!(f, "FileReader cannot find blob: {msg}"),
            FileReadError::NotReadable(msg) => {
                write!(f, "FileReader cannot read contents of blob: {msg}")
            }
            FileReadError::Security(msg) => {
                write!(f, "FileReader encountered a security exception: {msg}")
            }
        }
    }
}

impl std::error::Error for FileReadError {}
