#![deny(missing_docs, missing_debug_implementations)]

/*!
  XmlHttpRequest.
*/

/// The raw XmlHttpRequest (XHR) API, wrapped with convenient Rust types.
///
/// ### Example:
///
/// ```no_run
/// use gloo_xhr::callback::XmlHttpRequest;
/// use futures::sync::mpsc;
/// use gloo_events::EventListener;
///
/// let (sender, receiver) = mpsc::unbounded::<()>();
/// let request = XmlHttpRequest::new();
///
/// let load_listener = EventListener::new(request.as_ref(), "load", move |_event| {
///     sender.unbounded_send(()).unwrap();
/// });
///
/// load_listener.forget();
///
/// request.open(&http::Method::GET, "/");
///
/// request.send_without_body();
/// ```
///
/// ### Events
///
/// The [`XmlHttpRequest`](crate::callback::XmlHttpRequest) object is an event target. The most frequently used are:
///
/// - `load`: when the response is received
/// - `error`: when a network error occurred
/// - `progress`: when progress is made on the **response download**. For progress on the request
/// upload, see "Upload progress" below.
///
/// You can conveniently attach event listeners with [gloo-events][gloo-events].
///
/// ```no_run
/// use gloo_xhr::callback::XmlHttpRequest;
///
/// let request = XmlHttpRequest::new();
///
/// gloo_events::EventListener::new(request.as_ref(), "load", move |_event| {
///     // do something with the event
/// });
/// ```
///
/// ### Upload progress
///
/// XHR supports progress events on uploads. You can access these events by attaching an event
/// listener to the [`XmlHttpRequestUpload`](crate::callback::XmlHttpRequestUpload) object (returned by the `upload` method).
///
/// For further documentation on the browser API, see the [MDN guide][MDN using XHR].
///
/// ### Unsupported features
///
/// Synchronous requests are not supported because they are considered deprecated by most browsers.
///
/// You can access the underlying [`web_sys::XmlHttpRequest`](web_sys::XmlHttpRequest) through the [`as_raw`](crate::callback::XmlHttpRequest::as_raw) method.
///
/// [MDN using XHR]: https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/Using_XMLHttpRequest
/// [gloo-events]: https://github.com/rustwasm/gloo
pub mod callback {
    use wasm_bindgen::{prelude::*, JsCast};
    use web_sys;

    /// The main interface to the XmlHttpRequest API. See [module docs](crate::callback) for more extensive
    /// documentation.
    #[derive(Debug, Clone)]
    pub struct XmlHttpRequest {
        xhr: web_sys::XmlHttpRequest,
    }

    // This is so it can be used as an event target by gloo-events.
    impl AsRef<web_sys::EventTarget> for XmlHttpRequest {
        fn as_ref(&self) -> &web_sys::EventTarget {
            self.xhr.as_ref()
        }
    }

    impl XmlHttpRequest {
        /// Initialize an XmlHttpRequest. To actually perform the request, you will need to
        /// call at least `open()` and `send()`.
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

        /// Returns an event target for upload events.
        ///
        /// See the [module docs](crate::callback) and `XmlHttpRequestUpload` for more information.
        pub fn upload(&self) -> XmlHttpRequestUpload {
            XmlHttpRequestUpload {
                upload: self.xhr.upload().unwrap_throw(),
            }
        }

        /// The `XMLHttpRequest` method `open()` initializes a newly-created request, or
        /// re-initializes an existing one.
        ///
        /// [MDN docs](https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/open)
        pub fn open(&self, method: &http::Method, url: &str) {
            self.xhr.open(&method.to_string(), url).unwrap_throw()
        }

        /// Abort the request.
        ///
        /// This will fire an `abort` event on the `XmlHttpRequest`.
        pub fn abort(&self) {
            self.xhr.abort().unwrap_throw()
        }

        /// Perform the request.
        ///
        /// Thise returns immediately - attach events handlers to be notified of the
        /// progress and outcome of the request.
        pub fn send<B: XhrBody>(&self, body: SendBody<B>) {
            match body {
                SendBody::None => self.send_without_body(),
                SendBody::Body(body) => self.send_with_body(body),
            }
        }

        /// `send()` without a body.
        pub fn send_without_body(&self) {
            self.xhr
                .send()
                .expect_throw("Error sending request. Did you forget to call `open`?")
        }

        /// Actually send the request. In order to know the outcome, you have to attach
        /// `load`, `abort` and `error` event listeners.
        pub fn send_with_body<B: XhrBody>(&self, body: B) {
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

    /// Represents the valid states for the request body: either a valid body or none.
    #[derive(Debug)]
    pub enum SendBody<B: XhrBody> {
        /// No body.
        None,
        /// A valid XmlHttpRequest body (implementing the `XhrBody` trait).
        Body(B),
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

    /// [MDN docs](https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/readyState)
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

    /// The main interface to the XmlHttpRequestUpload API.
    ///
    /// This is mainly used as an event target for uploads (see [module docs](super)), and is
    /// constructed by the [`upload()`](crate::callback::XmlHttpRequest::upload) method on
    /// [XmlHttpRequest](crate::callback::XmlHttpRequest)).
    ///
    /// [MDN docs](https://developer.mozilla.org/en-US/docs/Web/API/XMLHttpRequest/upload)
    #[derive(Debug, Clone)]
    pub struct XmlHttpRequestUpload {
        upload: web_sys::XmlHttpRequestUpload,
    }

    // This is so it can be used as an event target by gloo-events.
    impl AsRef<web_sys::EventTarget> for XmlHttpRequestUpload {
        fn as_ref(&self) -> &web_sys::EventTarget {
            self.upload.as_ref()
        }
    }
}
