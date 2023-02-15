use std::{convert::From, fmt};

use crate::{js_to_error, Error};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

use crate::http::Headers;
#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
use serde::de::DeserializeOwned;

/// The [`Request`]'s response
pub struct Response(web_sys::Response);

impl Response {
    /// Returns an instance of response builder
    #[allow(clippy::new_ret_no_self)]
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::new()
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
    pub fn type_(&self) -> web_sys::ResponseType {
        self.0.type_()
    }

    /// The URL of the response.
    ///
    /// The returned value will be the final URL obtained after any redirects.
    pub fn url(&self) -> String {
        self.0.url()
    }

    /// Whether or not this response is the result of a request you made which was redirected.
    pub fn redirected(&self) -> bool {
        self.0.redirected()
    }

    /// the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) of the
    /// response.
    pub fn status(&self) -> u16 {
        self.0.status()
    }

    /// Whether the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
    /// was a success code (in the range `200 - 299`).
    pub fn ok(&self) -> bool {
        self.0.ok()
    }

    /// The status message corresponding to the
    /// [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) from
    /// `Response::status`.
    ///
    /// For example, this would be 'OK' for a status code 200, 'Continue' for 100, or 'Not Found'
    /// for 404.
    pub fn status_text(&self) -> String {
        self.0.status_text()
    }

    /// Gets the headers.
    pub fn headers(&self) -> Headers {
        Headers::from_raw(self.0.headers())
    }

    /// Has the response body been consumed?
    ///
    /// If true, then any future attempts to consume the body will error.
    pub fn body_used(&self) -> bool {
        self.0.body_used()
    }

    /// Gets the body.
    pub fn body(&self) -> Option<web_sys::ReadableStream> {
        self.0.body()
    }

    /// Reads the response to completion, returning it as `FormData`.
    pub async fn form_data(&self) -> Result<web_sys::FormData, Error> {
        let promise = self.0.form_data().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        Ok(web_sys::FormData::from(val))
    }

    /// Reads the response to completion, parsing it as JSON.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str::<T>(&self.text().await?).map_err(Error::from)
    }

    /// Reads the response as a String.
    pub async fn text(&self) -> Result<String, Error> {
        let promise = self.0.text().unwrap();
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        let string = js_sys::JsString::from(val);
        Ok(String::from(&string))
    }

    /// Gets the binary response
    ///
    /// This works by obtaining the response as an `ArrayBuffer`, creating a `Uint8Array` from it
    /// and then converting it to `Vec<u8>`
    pub async fn binary(&self) -> Result<Vec<u8>, Error> {
        let promise = self.0.array_buffer().map_err(js_to_error)?;
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

impl From<web_sys::Response> for Response {
    fn from(raw: web_sys::Response) -> Self {
        Self(raw)
    }
}

impl From<Response> for web_sys::Response {
    fn from(res: Response) -> Self {
        res.0
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

/// A writable wrapper around `web_sys::Reponse`: an http response to be used with the `fetch` API
/// on a server side javascript runtime
pub struct ResponseBuilder {
    headers: Headers,
    options: web_sys::ResponseInit,
    body: Option<ResponseBody>,
}

/// Possible initializers for request body
#[derive(Debug)]
pub enum ResponseBody {
    /// Blob response body
    Blob(web_sys::Blob),
    /// Buffer response body
    Buffer(js_sys::Object),
    /// `Uint8Array` response body
    U8(Vec<u8>),
    /// `FormData` response body
    Form(web_sys::FormData),
    /// `URLSearchParams` response body
    Search(web_sys::UrlSearchParams),
    /// String response body
    Str(String),
    /// ReadableStream response body
    Stream(web_sys::ReadableStream),
}

impl ResponseBuilder {
    /// Creates a new response object
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace _all_ the headers.
    pub fn headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Sets a header.
    pub fn header(self, key: &str, value: &str) -> Self {
        self.headers.set(key, value);
        self
    }

    /// Set the status code
    pub fn status(mut self, status: u16) -> Self {
        self.options.status(status);
        self
    }

    /// Set the status text
    pub fn status_text(mut self, status_text: &str) -> Self {
        self.options.status_text(status_text);
        self
    }

    /// A convenience method to set JSON as response body
    ///
    /// # Note
    ///
    /// This method also sets the `Content-Type` header to `application/json`
    pub fn json<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<Response, Error> {
        let json = serde_json::to_string(value)?;
        self.header("Content-Type", "application/json")
            .body(Some(ResponseBody::Str(json)))
    }

    /// Set the response body and return the response
    pub fn body(mut self, data: Option<ResponseBody>) -> Result<Response, Error> {
        self.body = data;

        use web_sys::Response as R;
        self.options.headers(&self.headers.into_raw());
        let init = &self.options;
        match self.body {
            None => R::new_with_opt_str_and_init(None, init),
            Some(x) => match x {
                ResponseBody::Blob(y) => R::new_with_opt_blob_and_init(Some(&y), init),
                ResponseBody::Buffer(y) => R::new_with_opt_buffer_source_and_init(Some(&y), init),
                ResponseBody::U8(mut y) => {
                    R::new_with_opt_u8_array_and_init(Some(y.as_mut_slice()), init)
                }
                ResponseBody::Form(y) => R::new_with_opt_form_data_and_init(Some(&y), init),
                ResponseBody::Search(y) => {
                    R::new_with_opt_url_search_params_and_init(Some(&y), init)
                }
                ResponseBody::Str(y) => R::new_with_opt_str_and_init(Some(&y), init),
                ResponseBody::Stream(y) => R::new_with_opt_readable_stream_and_init(Some(&y), init),
            },
        }
        .map(Response)
        .map_err(js_to_error)
    }
}

impl Default for ResponseBuilder {
    fn default() -> Self {
        Self {
            headers: Headers::new(),
            options: web_sys::ResponseInit::new(),
            body: None,
        }
    }
}

impl fmt::Debug for ResponseBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response Builder")
            .field("headers", &self.headers)
            .finish()
    }
}
