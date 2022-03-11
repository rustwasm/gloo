//! Wrapper around the `fetch` API.
//!
//! # Example
//!
//! ```
//! # use reqwasm::http::Request;
//! # async fn no_run() {
//! let resp = Request::get("/path")
//!     .send()
//!     .await
//!     .unwrap();
//! assert_eq!(resp.status(), 200);
//! # }
//! ```

use crate::{js_to_error, Error};
use js_sys::{ArrayBuffer, Uint8Array};
use std::fmt;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
use serde::de::DeserializeOwned;

pub use web_sys::{
    AbortSignal, FormData, ObserverCallback, ReadableStream, ReferrerPolicy, RequestCache,
    RequestCredentials, RequestMode, RequestRedirect,
};

#[allow(
    missing_docs,
    missing_debug_implementations,
    clippy::upper_case_acronyms
)]
/// Valid request methods.
#[derive(Clone, Copy, Debug)]
pub enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Method::GET => "GET",
            Method::HEAD => "HEAD",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::CONNECT => "CONNECT",
            Method::OPTIONS => "OPTIONS",
            Method::TRACE => "TRACE",
            Method::PATCH => "PATCH",
        };
        write!(f, "{}", s)
    }
}

/// A wrapper round `web_sys::Request`: an http request to be used with the `fetch` API.
pub struct Request {
    options: web_sys::RequestInit,
    headers: web_sys::Headers,
    url: String,
}

impl Request {
    /// Creates a new request that will be sent to `url`.
    ///
    /// Uses `GET` by default. `url` can be a `String`, a `&str`, or a `Cow<'a, str>`.
    pub fn new(url: &str) -> Self {
        Self {
            options: web_sys::RequestInit::new(),
            headers: web_sys::Headers::new().expect("headers"),
            url: url.into(),
        }
    }

    /// Set the body for this request.
    pub fn body(mut self, body: impl Into<JsValue>) -> Self {
        self.options.body(Some(&body.into()));
        self
    }

    /// A string indicating how the request will interact with the browser’s HTTP cache.
    pub fn cache(mut self, cache: RequestCache) -> Self {
        self.options.cache(cache);
        self
    }

    /// Controls what browsers do with credentials (cookies, HTTP authentication entries, and TLS
    /// client certificates).
    pub fn credentials(mut self, credentials: RequestCredentials) -> Self {
        self.options.credentials(credentials);
        self
    }

    /// Sets a header.
    pub fn header(self, key: &str, value: &str) -> Self {
        self.headers.set(key, value).expect("set header");
        self
    }

    /// The subresource integrity value of the request (e.g.,
    /// `sha256-BpfBw7ivV8q2jLiT13fxDYAe2tJllusRSZ273h2nFSE=`).
    pub fn integrity(mut self, integrity: &str) -> Self {
        self.options.integrity(integrity);
        self
    }

    /// The request method, e.g., GET, POST.
    ///
    /// Note that the Origin header is not set on Fetch requests with a method of HEAD or GET.
    pub fn method(mut self, method: Method) -> Self {
        self.options.method(&method.to_string());
        self
    }

    /// The mode you want to use for the request.
    pub fn mode(mut self, mode: RequestMode) -> Self {
        self.options.mode(mode);
        self
    }

    /// Sets the observer callback.
    pub fn observe(mut self, observe: &ObserverCallback) -> Self {
        self.options.observe(observe);
        self
    }

    /// How to handle a redirect response:
    ///
    /// - *follow*: Automatically follow redirects. Unless otherwise stated the redirect mode is
    ///   set to follow
    /// - *error*: Abort with an error if a redirect occurs.
    /// - *manual*: Caller intends to process the response in another context. See [WHATWG fetch
    ///   standard](https://fetch.spec.whatwg.org/#requests) for more information.
    pub fn redirect(mut self, redirect: RequestRedirect) -> Self {
        self.options.redirect(redirect);
        self
    }

    /// The referrer of the request.
    ///
    /// This can be a same-origin URL, `about:client`, or an empty string.
    pub fn referrer(mut self, referrer: &str) -> Self {
        self.options.referrer(referrer);
        self
    }

    /// Specifies the
    /// [referrer policy](https://w3c.github.io/webappsec-referrer-policy/#referrer-policies) to
    /// use for the request.
    pub fn referrer_policy(mut self, referrer_policy: ReferrerPolicy) -> Self {
        self.options.referrer_policy(referrer_policy);
        self
    }

    /// Sets the request abort signal.
    pub fn abort_signal(mut self, signal: Option<&AbortSignal>) -> Self {
        self.options.signal(signal);
        self
    }

    /// Executes the request.
    pub async fn send(mut self) -> Result<Response, Error> {
        self.options.headers(&self.headers);

        let request = web_sys::Request::new_with_str_and_init(&self.url, &self.options)
            .map_err(js_to_error)?;

        let promise = window().unwrap().fetch_with_request(&request);
        let response = JsFuture::from(promise).await.map_err(js_to_error)?;
        match response.dyn_into::<web_sys::Response>() {
            Ok(response) => Ok(Response {
                response: response.unchecked_into(),
            }),
            Err(e) => panic!("fetch returned {:?}, not `Response` - this is a bug", e),
        }
    }

    /// Creates a new [`GET`][Method::GET] `Request` with url.
    pub fn get(url: &str) -> Self {
        Self::new(url).method(Method::GET)
    }

    /// Creates a new [`POST`][Method::POST] `Request` with url.
    pub fn post(url: &str) -> Self {
        Self::new(url).method(Method::POST)
    }

    /// Creates a new [`PUT`][Method::PUT] `Request` with url.
    pub fn put(url: &str) -> Self {
        Self::new(url).method(Method::PUT)
    }

    /// Creates a new [`DELETE`][Method::DELETE] `Request` with url.
    pub fn delete(url: &str) -> Self {
        Self::new(url).method(Method::DELETE)
    }

    /// Creates a new [`PATCH`][Method::PATCH] `Request` with url.
    pub fn patch(url: &str) -> Self {
        Self::new(url).method(Method::PATCH)
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request").field("url", &self.url).finish()
    }
}

/// The [`Request`]'s response
pub struct Response {
    response: web_sys::Response,
}

impl Response {
    /// Downcast a js value to an instance of web_sys::Response.
    ///
    /// # Correctness
    ///
    /// Will result in incorrect code if `raw` is not a `web_sys::Response`.
    fn from_raw(raw: JsValue) -> Self {
        let raw: web_sys::Response = raw.unchecked_into();
        Self { response: raw }
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
        self.response.type_()
    }

    /// The URL of the response.
    ///
    /// The returned value will be the final URL obtained after any redirects.
    pub fn url(&self) -> String {
        self.response.url()
    }

    /// Whether or not this response is the result of a request you made which was redirected.
    pub fn redirected(&self) -> bool {
        self.response.redirected()
    }

    /// the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) of the
    /// response.
    pub fn status(&self) -> u16 {
        self.response.status()
    }

    /// Whether the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
    /// was a success code (in the range `200 - 299`).
    pub fn ok(&self) -> bool {
        self.response.ok()
    }

    /// The status message corresponding to the
    /// [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) from
    /// `Response::status`.
    ///
    /// For example, this would be 'OK' for a status code 200, 'Continue' for 100, or 'Not Found'
    /// for 404.
    pub fn status_text(&self) -> String {
        self.response.status_text()
    }

    /// Gets the headers.
    pub fn headers(&self) -> Headers {
        Headers::from_raw(self.response.headers())
    }

    /// Has the response body been consumed?
    ///
    /// If true, then any future attempts to consume the body will error.
    pub fn body_used(&self) -> bool {
        self.response.body_used()
    }

    /// Gets the body.
    pub fn body(&self) -> Option<ReadableStream> {
        self.response.body()
    }

    /// Gets the raw [`Response`][web_sys::Response] object.
    pub fn as_raw(&self) -> &web_sys::Response {
        &self.response
    }

    /// Reads the response to completion, returning it as `FormData`.
    pub async fn form_data(&self) -> Result<FormData, Error> {
        let promise = self.response.form_data().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        Ok(FormData::from(val))
    }

    /// Reads the response to completion, parsing it as JSON.
    ///
    /// An alternative here is to get the data as bytes (`body_as_vec`) or a string (`text`), and
    /// then parse the json in Rust, using `serde` or something else.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let promise = self.response.json().map_err(js_to_error)?;
        let json = JsFuture::from(promise).await.map_err(js_to_error)?;

        Ok(json.into_serde()?)
    }

    /// Reads the response as a String.
    pub async fn text(&self) -> Result<String, Error> {
        let promise = self.response.text().unwrap();
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        let string = js_sys::JsString::from(val);
        Ok(String::from(&string))
    }

    /// Gets the binary response
    ///
    /// This works by obtaining the response as an `ArrayBuffer`, creating a `Uint8Array` from it
    /// and then converting it to `Vec<u8>`
    pub async fn binary(&self) -> Result<Vec<u8>, Error> {
        let promise = self.response.array_buffer().map_err(js_to_error)?;
        let array_buffer: ArrayBuffer = JsFuture::from(promise)
            .await
            .map_err(js_to_error)?
            .unchecked_into();
        let typed_buff: Uint8Array = Uint8Array::new(&array_buffer);
        let mut body = vec![0; typed_buff.length() as usize];
        typed_buff.copy_to(&mut body);
        Ok(body)
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response")
            .field("url", &self.url())
            .field("redirected", &self.redirected())
            .field("status", &self.status())
            .field("headers", &self.headers())
            .field("body_used", &self.body_used())
            .finish()
    }
}
