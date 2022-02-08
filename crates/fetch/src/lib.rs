pub mod callback;

use std::borrow::Cow;
use wasm_bindgen::JsValue;
use web_sys::RequestInit;

pub use http;
pub use web_sys::{RequestCache, RequestMode};

pub struct Request<'a> {
    url: Cow<'a, str>,
    init: RequestInit,
    headers: http::HeaderMap,
}

impl<'a> Request<'a> {
    /// `url` can be a `String`, a `&str`, or a `Cow<'a, str>`.
    pub fn new(url: impl Into<Cow<'a, str>>) -> Self {
        Self {
            url: url.into(),
            init: RequestInit::new(),
            headers: http::HeaderMap::new(),
        }
    }

    pub fn get(url: impl Into<Cow<'a, str>>) -> Self {
        let mut req = Self::new(url);
        // GET is the default.
        //req.method(http::Method::GET);
        req
    }

    pub fn post(url: impl Into<Cow<'a, str>>, body: impl RequestBody) -> Self {
        let mut req = Self::new(url);
        req.method(http::Method::POST);
        req.init.body(Some(&body.as_js_value()));
        req
    }

    /// The request method, e.g., GET, POST.
    ///
    /// Note that the Origin header is not set on Fetch requests with a method of HEAD or GET.
    pub fn method(&mut self, method: http::Method) -> &mut Self {
        self.init.method(method.as_str());
        self
    }

    /// Set the content type for the request (e.g. `application/json` for json, `text/html` for
    /// Html)
    ///
    /// # Panics
    ///
    /// Panics if the content type contains any invalid bytes (`<32` apart from tab, and `127`).
    pub fn content_type(&mut self, content_type: impl AsRef<[u8]>) -> &mut Self {
        self.insert_header(
            "Content-Type",
            http::HeaderValue::from_bytes(content_type.as_ref()).expect("invalid content type"),
        );
        self
    }

    /// Add a header to the request, replacing an existing header with the same name.
    ///
    /// Note that
    /// [some names are forbidden](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name).
    pub fn insert_header(
        &mut self,
        name: impl http::header::IntoHeaderName,
        value: impl Into<http::HeaderValue>,
    ) -> &mut Self {
        self.headers.insert(name, value.into());
        self
    }

    /// Add a header to the request, adding a duplicate if an existing header has the same name.
    ///
    /// Note that
    /// [some names are forbidden](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name).
    pub fn append_header(
        &mut self,
        name: impl http::header::IntoHeaderName,
        value: impl Into<http::HeaderValue>,
    ) -> &mut Self {
        self.headers.append(name, value.into());
        self
    }

    pub fn headers(&self) -> &http::HeaderMap {
        &self.headers
    }

    pub fn headers_mut(&mut self) -> &mut http::HeaderMap {
        &mut self.headers
    }

    /// The subresource integrity value of the request (e.g.,
    /// `sha256-BpfBw7ivV8q2jLiT13fxDYAe2tJllusRSZ273h2nFSE=`).
    pub fn integrity(&mut self, integrity: &'_ str) -> &mut Self {
        self.init.integrity(integrity);
        self
    }

    /// The mode you want to use for the request.
    pub fn request_mode(&mut self, mode: RequestMode) -> &mut Self {
        self.init.mode(mode);
        self
    }
}

trait Sealed {}

pub trait RequestBody: Sealed {
    fn as_js_value(&self) -> JsValue;
}

impl Sealed for String {}
impl RequestBody for String {
    fn as_js_value(&self) -> JsValue {
        JsValue::from_str(&self)
    }
}

// TODO
