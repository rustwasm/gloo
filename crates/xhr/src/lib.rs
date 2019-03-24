#![deny(missing_docs, missing_debug_implementations)]

/*!
  XmlHttpRequest.
*/

/// The raw XmlHttpRequest API, wrapped with convenient Rust types.
///
/// Synchronous requests are not supported (because deprecated).
pub mod raw {
    use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
    use web_sys;

    /// See MDN docs.
    #[derive(Debug, Clone)]
    pub struct XmlHttpRequest {
        xhr: web_sys::XmlHttpRequest,
    }

    impl XmlHttpRequest {
        /// Initialize an XmlHttpRequest.
        pub fn new() -> Self {
            // we assume this is safe because all browsers that support webassembly
            // implement XmlHttpRequest.
            let xhr = web_sys::XmlHttpRequest::new().unwrap_throw();
            XmlHttpRequest { xhr }
        }

        /// Access the underlying raw `web_sys::XmlHttpRequest`.
        pub fn as_raw(&self) -> &web_sys::XmlHttpRequest {
            &self.xhr
        }

        /// See MDN docs.
        ///
        /// Upload events are fired there.
        pub fn upload(&self) -> XmlHttpRequestUpload {
            XmlHttpRequestUpload {
                upload: self.xhr.upload().unwrap_throw(),
            }
        }

        /// Open?
        pub fn open(&self, method: &http::Method, url: &str) {
            self.xhr
                .open(&method.to_string(), url)
                .expect_throw("opening XHR")
        }

        /// Abort the request.
        pub fn abort(&self) {
            self.xhr.abort().expect_throw("aborting XHR")
        }

        /// Send without a body.
        ///
        /// Should probably be renamed.
        pub fn send_no_body(&self) {
            self.xhr
                .send()
                .expect_throw("Error sending request. Did you forget to call `open`?")
        }

        /// Send!
        pub fn send<B: XhrBody>(&self, body: B) {
            body.send(&self.xhr)
                .expect_throw("Error sending request. Did you forget to call `open`?")
        }

        /// Set a header on the request.
        pub fn set_request_header(&self, header: &str, value: &str) {
            self.xhr
                .set_request_header(header, value)
                .expect_throw("XHR header can be set")
        }

        /// Set the response timeout.
        pub fn set_timeout(&self, timeout: std::time::Duration) {
            self.xhr.set_timeout(timeout.as_millis() as u32)
        }

        /// Get a single header on the response.
        pub fn get_response_header(&self, header: &str) -> Option<String> {
            self.xhr.get_response_header(header).unwrap_throw()
        }

        /// Get all the response headers.
        pub fn get_all_response_headers(&self) -> std::collections::HashMap<String, String> {
            self.xhr
                .get_all_response_headers()
                .expect_throw("XMLHttpRequest.getAllResponseHeaders() error")
                .split("\r\n")
                .map(|line| {
                    let mut elems = line.split(": ");
                    (
                        elems.next().unwrap_throw().to_string(),
                        elems.next().unwrap_throw().to_string(),
                    )
                })
                .collect()
        }

        /// The error callback can fire in cases such as CORS errors.
        pub fn set_onerror<C>(&self, callback: C)
        where
            C: FnMut(web_sys::ProgressEvent) + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<FnMut(web_sys::ProgressEvent)>);
            self.xhr.set_onerror(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        /// see mdn
        ///
        /// This takes an FnMut because the callback can be called more than once (if
        /// `send` is called more than once)
        pub fn set_onload<C>(&self, callback: C)
        where
            C: FnMut(web_sys::ProgressEvent) + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<FnMut(web_sys::ProgressEvent)>);
            self.xhr.set_onload(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        /// see mdn
        pub fn set_onprogress<C>(&self, callback: C)
        where
            C: FnMut(web_sys::ProgressEvent) + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<FnMut(web_sys::ProgressEvent)>);
            self.xhr
                .set_onprogress(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        /// Get the response body, assuming `responseType` was set to text (the default).
        pub fn response_as_string(&self) -> Option<String> {
            // unwrap_throw is safe, because `response` is an accessor. If the response
            // is not present yet, it will be `null` (`None`).
            self.xhr.response().unwrap_throw().as_string()
        }

        /// Get the response body, assuming `responseType` was set to `"arraybuffer"`.
        pub fn response_as_array_buffer(&self) -> Option<js_sys::ArrayBuffer> {
            // unwrap_throw is safe, because `response` is an accessor. If the response
            // is not present yet, it will be `null` (`None`).
            self.xhr.response().unwrap_throw().dyn_into().ok()
        }

        /// Get the response body, assuming `responseType` was set to `"blob"`.
        pub fn response_as_blob(&self) -> Option<web_sys::Blob> {
            // unwrap_throw is safe, because `response` is an accessor. If the response
            // is not present yet, it will be `null` (`None`).
            self.xhr.response().unwrap_throw().dyn_into().ok()
        }

        /// Get the response body, assuming `responseType` was set to `"document"`.
        pub fn response_as_document(&self) -> Option<web_sys::Document> {
            // unwrap_throw is safe, because `response` is an accessor. If the response
            // is not present yet, it will be `null` (`None`).
            self.xhr.response().unwrap_throw().dyn_into().ok()
        }

        /// Get the response body as a byte buffer, assuming `responseType` was set to
        /// "arraybuffer".
        pub fn response_as_bytes(&self) -> Option<Vec<u8>> {
            let array_buffer = self.response_as_array_buffer()?;
            let byte_array = js_sys::Uint8Array::new(&array_buffer);
            let mut dest_buffer = Vec::<u8>::with_capacity(byte_array.length() as usize);
            byte_array.copy_to(&mut dest_buffer);
            Some(dest_buffer)
        }

        /// Get the `ReadyState` of the request.
        ///
        /// https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/readyState
        pub fn ready_state(&self) -> ReadyState {
            ReadyState::from_u16(self.xhr.ready_state())
                .expect_throw("XMLHttpRequest ReadyState must be 0 ≤ n ≤ 4")
        }
    }

    /// This trait is implemented by all the types that can be used as the body of a
    /// request constructed via XmlHttpRequest.
    ///
    /// Users should use `XmlHttpRequest::set_body` rather than this trait directly.
    pub trait XhrBody {
        /// Send the XmlHttpRequest with its body set to `self`.
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue>;
    }

    impl XhrBody for &web_sys::FormData {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            xhr.send_with_opt_form_data(Some(self))
        }
    }

    impl XhrBody for &mut [u8] {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            xhr.send_with_opt_u8_array(Some(self))
        }
    }

    impl XhrBody for &web_sys::Document {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            xhr.send_with_opt_document(Some(self))
        }
    }

    impl XhrBody for &str {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            xhr.send_with_opt_str(Some(self))
        }
    }

    impl XhrBody for &web_sys::Blob {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            xhr.send_with_opt_blob(Some(self))
        }
    }

    impl XhrBody for &web_sys::File {
        fn send(self, xhr: &web_sys::XmlHttpRequest) -> Result<(), JsValue> {
            let blob: &web_sys::Blob = self.unchecked_ref();
            xhr.send_with_opt_blob(Some(blob))
        }
    }

    /// https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/responseType
    ///
    /// Attempts to set the value of responseType to "document" are ignored in a Worker.
    ///
    /// TODO: wrap the web_sys version instead
    #[derive(Debug)]
    pub enum ResponseType {
        /// ArrayBuffer
        ArrayBuffer,
        /// Blob
        Blob,
        /// Document
        ///
        /// Attempts to set the value of responseType to Document are ignored in a Worker.
        Document,
        /// Not supported in Edge and IE as of March 2019.
        ///
        /// See compatibility table: https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/responseType
        Json,
        /// The default response type.
        Text,
    }

    /// https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/readyState
    #[derive(Debug)]
    pub enum ReadyState {
        /// Client has been created. `open()` not called yet.
        Unsent,
        /// `open()` has been called.
        Opened,
        /// `send()` has been called, and headers and status are available.
        HeadersReceived,
        /// Downloading; `responseText` holds partial data.
        Loading,
        /// The operation is complete.
        Done,
    }

    impl ReadyState {
        /// Convert to the integer status code.
        pub fn to_u16(&self) -> u16 {
            match self {
                ReadyState::Unsent => 0,
                ReadyState::Opened => 1,
                ReadyState::HeadersReceived => 2,
                ReadyState::Loading => 3,
                ReadyState::Done => 4,
            }
        }

        /// Convert from the integer status code.
        pub fn from_u16(status_code: u16) -> Option<ReadyState> {
            match status_code {
                0 => Some(ReadyState::Unsent),
                1 => Some(ReadyState::Opened),
                2 => Some(ReadyState::HeadersReceived),
                3 => Some(ReadyState::Loading),
                4 => Some(ReadyState::Done),
                _ => None,
            }
        }
    }

    /// See MDN docs.
    #[derive(Debug, Clone)]
    pub struct XmlHttpRequestUpload {
        upload: web_sys::XmlHttpRequestUpload,
    }

    impl XmlHttpRequestUpload {
        /// see mdn
        pub fn set_onload<C>(&self, callback: C)
        where
            C: FnMut(web_sys::ProgressEvent) + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<FnMut(web_sys::ProgressEvent)>);
            self.upload
                .set_onload(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }

        /// see mdn
        pub fn set_onprogress<C>(&self, callback: C)
        where
            C: FnMut(web_sys::ProgressEvent) + 'static,
        {
            let closure = Closure::wrap(Box::new(callback) as Box<FnMut(web_sys::ProgressEvent)>);
            self.upload
                .set_onprogress(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
    }
}
