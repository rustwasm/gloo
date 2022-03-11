//! A library that wraps the HTTP *fetch* API.
#![warn(missing_docs)]

mod headers;

use js_sys::{ArrayBuffer, Promise, Uint8Array};
use std::{
    cell::{RefCell, RefMut},
    ops::Deref,
};
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;

pub use crate::headers::Headers;
#[doc(inline)]
pub use web_sys::{
    ReferrerPolicy, RequestCache, RequestCredentials, RequestMode, RequestRedirect, ResponseType,
};

/// A wrapper round `web_sys::Request`: an http request to be used with the `fetch` API.
pub struct Request {
    init: RefCell<web_sys::RequestInit>,
    headers: RefCell<Option<Headers>>,
}

impl Request {
    /// Creates a new request that will be sent to `url`.
    ///
    /// Uses `GET` by default. `url` can be a `String`, a `&str`, or a `Cow<'a, str>`.
    pub fn new() -> Self {
        Self {
            init: RefCell::new(web_sys::RequestInit::new()),
            headers: RefCell::new(None),
        }
    }

    /// The request method, e.g., GET, POST.
    ///
    /// Note that the Origin header is not set on Fetch requests with a method of HEAD or GET.
    pub fn method(&mut self, method: &str) -> &mut Self {
        self.init.borrow_mut().method(method);
        self
    }

    /// Get access to the headers object for this request.
    ///
    /// If you are going to insert new headers, note that
    /// [some names are forbidden](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name)
    /// and if they are set then sending the request will error.
    pub fn headers(&mut self) -> impl Deref<Target = Headers> + '_ {
        RefMut::map(self.headers.borrow_mut(), |opt| {
            opt.get_or_insert_with(|| Headers::new())
        })
    }

    /// Set the body for this request.
    pub fn body(&mut self, body: impl RequestBody) -> &mut Self {
        self.init.borrow_mut().body(body.as_js_value().as_ref());
        self
    }

    /// Set the content type for the request (e.g. `application/json` for json, `text/html` for
    /// Html).
    ///
    /// If you want a more typed experience, you can use the
    /// [`mime`](https://crates.io/crates/mime) crate, and use the `impl Display for mime::Mime` to
    /// get a byte string.
    ///
    /// # Panics
    ///
    /// Panics if the content type contains any invalid bytes (`<32` apart from tab, and `127`).
    #[doc(alias("mime_type", "media_type"))]
    pub fn content_type(&mut self, content_type: &str) -> &mut Self {
        self.headers().set("Content-Type", content_type);
        self
    }

    /// A string indicating how the request will interact with the browser’s HTTP cache.
    pub fn cache(&mut self, cache: RequestCache) -> &mut Self {
        self.init.borrow_mut().cache(cache);
        self
    }

    /// Controls what browsers do with credentials (cookies, HTTP authentication entries, and TLS
    /// client certificates).
    pub fn credentials(&mut self, credentials: RequestCredentials) -> &mut Self {
        self.init.borrow_mut().credentials(credentials);
        self
    }

    /// The subresource integrity value of the request (e.g.,
    /// `sha256-BpfBw7ivV8q2jLiT13fxDYAe2tJllusRSZ273h2nFSE=`).
    pub fn integrity(&mut self, integrity: &'_ str) -> &mut Self {
        self.init.borrow_mut().integrity(integrity);
        self
    }

    /// The mode you want to use for the request.
    pub fn mode(&mut self, mode: RequestMode) -> &mut Self {
        self.init.borrow_mut().mode(mode);
        self
    }

    /// How to handle a redirect response:
    ///
    /// - *follow*: Automatically follow redirects. Unless otherwise stated the redirect mode is
    ///   set to follow
    /// - *error*: Abort with an error if a redirect occurs.
    /// - *manual*: Caller intends to process the response in another context. See [WHATWG fetch
    ///   standard](https://fetch.spec.whatwg.org/#requests) for more information.
    pub fn redirect(&mut self, redirect: RequestRedirect) -> &mut Self {
        self.init.borrow_mut().redirect(redirect);
        self
    }

    /// The referrer of the request.
    ///
    /// This can be a same-origin URL, `about:client`, or an empty string.
    pub fn referrer(&mut self, referrer: &'_ str) -> &mut Self {
        self.init.borrow_mut().referrer(referrer);
        self
    }

    /// Specifies the
    /// [referrer policy](https://w3c.github.io/webappsec-referrer-policy/#referrer-policies) to
    /// use for the request.
    pub fn referrer_policy(&mut self, policy: ReferrerPolicy) -> &mut Self {
        self.init.borrow_mut().referrer_policy(policy);
        self
    }

    fn apply_headers(&self) {
        if let Some(headers) = self.headers.borrow_mut().take() {
            self.init.borrow_mut().headers(&headers.raw);
        }
    }

    /// Send the request, returning a future that will resolve to the response once all headers
    /// have been received.
    pub async fn send(&self, url: impl AsRef<str>) -> Result<Response, JsValue> {
        let response = JsFuture::from(self.send_raw(url)).await?;
        Ok(Response::from_raw(response))
    }

    /// A wrapper round `fetch` to de-duplicate some boilerplate
    fn send_raw(&self, url: impl AsRef<str>) -> Promise {
        self.apply_headers();
        web_sys::window()
            .expect("no window")
            .fetch_with_str_and_init(url.as_ref(), &*self.init.borrow())
    }
}

mod private {
    pub trait Sealed {}
}

/// A trait for types that can be passed as the `body` to a `Request`.
///
/// This trait is sealed because we know all the types that are allowed and want to prevent
/// implementation for other types.
pub trait RequestBody: private::Sealed {
    /// `web_sys::Request::body` takes an untyped `JsValue`.
    ///
    /// This is an implementation detail - you shouldn't need to look at this trait at all.
    fn as_js_value(&self) -> Option<JsValue>;
}

impl private::Sealed for String {}
impl RequestBody for String {
    fn as_js_value(&self) -> Option<JsValue> {
        Some(JsValue::from_str(&self))
    }
}

impl<'a> private::Sealed for &'a str {}
impl<'a> RequestBody for &'a str {
    fn as_js_value(&self) -> Option<JsValue> {
        Some(JsValue::from_str(self))
    }
}

// TODO Blob, BufferSource, FormData, URLSearchParams, (USVString - done), ReadableStream

/// The response to a `fetch` request once all headers have been successfully received.
///
/// Note that the full response might not be received that this point, which is why methods that
/// access the response body are asynchronous.
pub struct Response {
    raw: web_sys::Response,
}

impl Response {
    /// Downcast a js value to an instance of web_sys::Response.
    ///
    /// # Correctness
    ///
    /// Will result in incorrect code if `raw` is not a `web_sys::Response`.
    fn from_raw(raw: JsValue) -> Self {
        let raw: web_sys::Response = raw.unchecked_into();
        Self { raw }
    }

    /// The type read-only property of the Response interface contains the type of the response.
    ///
    /// It can be one of the following:
    ///
    ///  - basic: Normal, same origin response, with all headers exposed except “Set-Cookie” and
    ///    “Set-Cookie2″.
    ///  - cors: Response was received from a valid cross-origin request. Certain headers and the
    ///    body may be accessed.
    ///  - error: Network error. No useful information describing the error is available. The
    ///    Response’s status is 0, headers are empty and immutable. This is the type for a Response
    ///    obtained from Response.error().
    ///  - opaque: Response for “no-cors” request to cross-origin resource. Severely restricted.
    ///  - opaqueredirect: The fetch request was made with redirect: "manual". The Response's
    ///    status is 0, headers are empty, body is null and trailer is empty.
    pub fn type_(&self) -> ResponseType {
        self.raw.type_()
    }

    /// The URL of the response.
    ///
    /// The returned value will be the final URL obtained after any redirects.
    pub fn url(&self) -> String {
        self.raw.url()
    }

    /// Whether or not this response is the result of a request you made which was redirected.
    pub fn redirected(&self) -> bool {
        self.raw.redirected()
    }

    /// the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) of the
    /// response.
    pub fn status(&self) -> u16 {
        self.raw.status()
    }

    /// Whether the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
    /// was a success code (in the range `200 - 299`).
    pub fn ok(&self) -> u16 {
        self.raw.status()
    }

    /// The status message corresponding to the
    /// [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) from
    /// `Response::status`.
    ///
    /// For example, this would be 'OK' for a status code 200, 'Continue' for 100, or 'Not Found'
    /// for 404.
    pub fn status_text(&self) -> String {
        self.raw.status_text()
    }

    /// Provides access to the headers contained in the response.
    ///
    /// Some headers may be inaccessible, depending on CORS and other things.
    pub fn headers(&self) -> Headers {
        Headers::from_raw(self.raw.headers())
    }

    /// Has the response body been consumed?
    ///
    /// If true, then any future attempts to consume the body will error.
    pub fn body_used(&self) -> bool {
        self.raw.body_used()
    }

    // TODO unsure how to handle streams, and personally don't need this functionality

    /// Reads the response to completion, returning it as an `ArrayBuffer`.
    pub async fn array_buffer(&self) -> Result<ArrayBuffer, JsValue> {
        JsFuture::from(self.raw.array_buffer().unwrap_throw())
            .await
            .map(JsCast::unchecked_into)
    }

    /// Reads the response to completion, returning it as a `Blob`.
    pub async fn blob(&self) -> Result<web_sys::Blob, JsValue> {
        JsFuture::from(self.raw.array_buffer().unwrap_throw())
            .await
            .map(JsCast::unchecked_into)
    }

    /// Reads the response to completion, returning it as `FormData`.
    pub async fn form_data(&self) -> Result<web_sys::FormData, JsValue> {
        JsFuture::from(self.raw.array_buffer().unwrap_throw())
            .await
            .map(JsCast::unchecked_into)
    }

    /// Reads the response to completion, parsing it as JSON.
    ///
    /// An alternative here is to get the data as bytes (`body_as_vec`) or a string (`text`), and
    /// then parse the json in Rust, using `serde` or something else.
    pub async fn json(&self) -> Result<JsValue, JsValue> {
        JsFuture::from(self.raw.array_buffer().unwrap_throw()).await
    }

    /// Reads the response as a String.
    pub async fn text(&self) -> Result<String, JsValue> {
        JsFuture::from(self.raw.text().unwrap_throw())
            .await
            .map(|ok| ok.as_string().unwrap_throw())
    }

    /// Reads the response into a `Vec<u8>`.
    pub async fn body_as_vec(&self) -> Result<Vec<u8>, JsValue> {
        let buf = self.array_buffer().await?;
        Ok(Uint8Array::new(&buf).to_vec())
    }
}
