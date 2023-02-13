use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

use crate::{js_to_error, Error};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{FormData, ReadableStream, ResponseType};

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
use serde::de::DeserializeOwned;

use crate::http::Headers;

/// The [`Request`]'s response
pub struct Response {
    raw: web_sys::Response,
}

impl Response {
    /// Returns an instance of response builder
    pub fn new() -> Builder {
        Builder::new()
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
    pub fn ok(&self) -> bool {
        self.raw.ok()
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

    /// Gets the headers.
    pub fn headers(&self) -> Headers {
        Headers::from_raw(self.raw.headers())
    }

    /// Has the response body been consumed?
    ///
    /// If true, then any future attempts to consume the body will error.
    pub fn body_used(&self) -> bool {
        self.raw.body_used()
    }

    /// Gets the body.
    pub fn body(&self) -> Option<ReadableStream> {
        self.raw.body()
    }

    /// Reads the response to completion, returning it as `FormData`.
    pub async fn form_data(&self) -> Result<FormData, Error> {
        let promise = self.raw.form_data().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        Ok(FormData::from(val))
    }

    /// Reads the response to completion, parsing it as JSON.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub async fn json<T: DeserializeOwned>(&self) -> Result<T, Error> {
        serde_json::from_str::<T>(&self.text().await?).map_err(Error::from)
    }

    /// Reads the response as a String.
    pub async fn text(&self) -> Result<String, Error> {
        let promise = self.raw.text().unwrap();
        let val = JsFuture::from(promise).await.map_err(js_to_error)?;
        let string = js_sys::JsString::from(val);
        Ok(String::from(&string))
    }

    /// Gets the binary response
    ///
    /// This works by obtaining the response as an `ArrayBuffer`, creating a `Uint8Array` from it
    /// and then converting it to `Vec<u8>`
    pub async fn binary(&self) -> Result<Vec<u8>, Error> {
        let promise = self.raw.array_buffer().map_err(js_to_error)?;
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

impl TryFrom<web_sys::Response> for Response {
    type Error = crate::Error;

    fn try_from(raw: web_sys::Response) -> Result<Response, Error> {
        Ok(Self { raw })
    }
}

impl TryInto<web_sys::Response> for Response {
    type Error = crate::Error;

    fn try_into(self) -> Result<web_sys::Response, Error> {
        Ok(self.raw)
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
pub struct Builder {
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
    Form(FormData),
    /// `URLSearchParams` response body
    Search(web_sys::UrlSearchParams),
    /// String response body
    Str(String),
    /// ReadableStream response body
    Stream(web_sys::ReadableStream),
}

impl Builder {
    /// Creates a new response object
    pub fn new() -> Self {
        Self {
            headers: Headers::new(),
            options: web_sys::ResponseInit::new(),
            body: None,
        }
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
        .map(|raw| Response { raw })
        .map_err(js_to_error)
    }
}

impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Response Builder")
            .field("headers", &self.headers)
            .finish()
    }
}
