use crate::http::{Headers, QueryParams, Response};
use crate::{js_to_error, Error};
use http::Method;
use js_sys::{ArrayBuffer, Reflect, Uint8Array};
use std::convert::{From, TryFrom, TryInto};
use std::fmt;
use std::str::FromStr;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    AbortSignal, FormData, ObserverCallback, ReadableStream, ReferrerPolicy, RequestCache,
    RequestCredentials, RequestMode, RequestRedirect,
};

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
use serde::de::DeserializeOwned;

/// A builder for [`Request`].
pub struct RequestBuilder {
    options: web_sys::RequestInit,
    headers: Headers,
    query: QueryParams,
    url: String,
}

impl RequestBuilder {
    /// Creates a new request that will be sent to `url`.
    ///
    /// Uses `GET` by default. `url` can be a `String`, a `&str`, or a `Cow<'a, str>`.
    pub fn new(url: &str) -> Self {
        Self {
            options: web_sys::RequestInit::new(),
            headers: Headers::new(),
            query: QueryParams::new(),
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

    /// Replace _all_ the headers.
    pub fn headers(mut self, headers: Headers) -> Self {
        self.headers = headers;
        self
    }

    /// Sets a header.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.set(key, value);
        self
    }

    /// Append query parameters to the url, given as `(name, value)` tuples. Values can be of any
    /// type that implements [`ToString`].
    ///
    /// It is possible to append the same parameters with the same name multiple times, so
    /// `.query([("a", "1"), ("a", "2")])` results in the query string `a=1&a=2`.
    ///
    /// # Examples
    ///
    /// The query parameters can be passed in various different forms:
    ///
    /// ```
    /// # fn no_run() {
    /// use std::collections::HashMap;
    /// use gloo_net::http::Request;
    ///
    /// let slice_params = [("key", "value")];
    /// let vec_params = vec![("a", "3"), ("b", "4")];
    /// let mut map_params: HashMap<&'static str, &'static str> = HashMap::new();
    /// map_params.insert("key", "another_value");
    ///
    /// let r = Request::get("/search")
    ///     .query(slice_params)
    ///     .query(vec_params)
    ///     .query(map_params);
    /// // Result URL: /search?key=value&a=3&b=4&key=another_value
    /// # }
    /// ```
    pub fn query<T, K, V>(mut self, params: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.query.extend(params);
        self
    }

    /// The subresource integrity value of the request (e.g.,
    /// `sha256-BpfBw7ivV8q2jLiT13fxDYAe2tJllusRSZ273h2nFSE=`).
    pub fn integrity(mut self, integrity: &str) -> Self {
        self.options.integrity(integrity);
        self
    }

    /// A convenience method to set JSON as request body.
    /// 
    /// # Note
    /// 
    /// This method also sets the `Content-Type` header to `application/json`
    /// 
    /// # Errors
    /// 
    /// This method will return an error if the value cannot be serialized
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json<T: serde::Serialize + ?Sized>(self, value: &T) -> Result<Self, Error> {
        let json = serde_json::to_string(value)?;
        Ok(self.header("Content-Type", "application/json").body(json))
    }

    /// The request method, e.g., GET, POST.
    pub fn method(mut self, method: Method) -> Self {
        self.options.method(method.as_ref());
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

    /// Builds the request and send it to the server, returning the received response.
    pub async fn send(self) -> Result<Response, Error> {
        let req: Request = self.try_into()?;
        req.send().await
    }

    /// Builds the request.
    pub fn build(self) -> Result<Request, Error> {
        self.try_into()
    }
}

impl TryFrom<RequestBuilder> for Request {
    type Error = crate::Error;

    fn try_from(mut value: RequestBuilder) -> Result<Self, Self::Error> {
        // To preserve existing query parameters of self.url, it must be parsed and extended with
        // self.query's parameters. As web_sys::Url just accepts absolute URLs, retrieve the
        // absolute URL through creating a web_sys::Request object.
        let request = web_sys::Request::new_with_str(&value.url).map_err(js_to_error)?;
        let url = web_sys::Url::new(&request.url()).map_err(js_to_error)?;
        let combined_query = match url.search().as_str() {
            "" => value.query.to_string(),
            _ => format!("{}&{}", url.search(), value.query),
        };
        url.set_search(&combined_query);

        let final_url = String::from(url.to_string());
        value.options.headers(&value.headers.into_raw());
        let request = web_sys::Request::new_with_str_and_init(&final_url, &value.options)
            .map_err(js_to_error)?;

        Ok(Request::from_raw(request))
    }
}

impl fmt::Debug for RequestBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request").field("url", &self.url).finish()
    }
}

/// The [`Request`] sent to the server
pub struct Request(web_sys::Request);

impl Request {
    /// Creates a new [`Request`] from a [`web_sys::Request`].
    /// 
    /// # Note
    /// 
    /// If the body of the request has already been read, other body readers will misbehave.
    pub fn from_raw(request: web_sys::Request) -> Self {
        Self(request)
    }

    /// Returns the underlying [`web_sys::Request`].
    pub fn into_raw(self) -> web_sys::Request {
        self.0
    }

    /// Creates a new GET [`RequestBuilder`] with url.
    pub fn get(url: &str) -> RequestBuilder {
        RequestBuilder::new(url).method(Method::GET)
    }

    /// Creates a new POST [`RequestBuilder`] with url.
    pub fn post(url: &str) -> RequestBuilder {
        RequestBuilder::new(url).method(Method::POST)
    }

    /// Creates a new PUT [`RequestBuilder`] with url.
    pub fn put(url: &str) -> RequestBuilder {
        RequestBuilder::new(url).method(Method::PUT)
    }

    /// Creates a new Delete [`RequestBuilder`] with url.
    pub fn delete(url: &str) -> RequestBuilder {
        RequestBuilder::new(url).method(Method::DELETE)
    }

    /// Creates a new PATCH [`RequestBuilder`] with url.
    pub fn patch(url: &str) -> RequestBuilder {
        RequestBuilder::new(url).method(Method::PATCH)
    }

    /// The URL of the request.
    pub fn url(&self) -> String {
        self.0.url()
    }

    /// Gets the headers.
    pub fn headers(&self) -> Headers {
        Headers::from_raw(self.0.headers())
    }

    /// Return the read only mode for the request
    pub fn mode(&self) -> RequestMode {
        self.0.mode()
    }

    /// Return the parsed method for the request
    pub fn method(&self) -> Method {
        Method::from_str(self.0.method().as_str()).unwrap()
    }

    /// Has the request body been consumed?
    ///
    /// If true, then any future attempts to consume the body will panic.
    /// 
    /// # Note
    /// 
    /// In normal usage, this should always return false. The body is only consumed
    /// by methods that take ownership of the request. However, if you manually
    /// build a [`Request`] from [`web_sys::Request`], then this could be true.
    pub fn body_used(&self) -> bool {
        self.0.body_used()
    }

    /// Returns the underlying body of the request.
    /// 
    /// # Note
    /// 
    /// This consumes the request, if you need to access the body multiple times,
    /// you should `clone` the request first.
    pub fn body(self) -> Option<ReadableStream> {
        self.0.body()
    }

    /// Returns the underlying body of the request as [`web_sys::FormData`].
    /// 
    /// # Note
    /// 
    /// This consumes the request, if you need to access the body multiple times,
    /// you should `clone` the request first.
    /// 
    /// # Errors
    /// 
    /// Throws a "TypeError" if the content type of the request is not `"multipart/form-data"` or 
    /// `"application/x-www-form-urlencoded"`.
    /// 
    /// Throws a "TypeError" if the body cannot be converted to [`web_sys::FormData`].
    pub async fn form_data(self) -> Result<FormData, Error> {
        let promise = self.0.form_data().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?; // should never fail?
        Ok(FormData::from(val))
    }

    /// Returns the underlying body as a string.
    /// 
    /// # Note
    /// 
    /// This consumes the request, if you need to access the body multiple times,
    /// you should `clone` the request first.
    /// 
    /// # Errors
    /// 
    /// This will return an error if the body cannot be decoded as utf-8.
    pub async fn text(self) -> Result<String, Error> {
        let promise = self.0.text().map_err(js_to_error)?;
        let val = JsFuture::from(promise).await.map_err(js_to_error)?; // should never fail?
        let string = js_sys::JsString::from(val);
        Ok(String::from(&string))
    }

    /// Returns the underlying body of the request and parses it as JSON.
    /// 
    /// # Note
    /// 
    /// This consumes the request, if you need to access the body multiple times,
    /// you should `clone` the request first.
    /// 
    /// # Errors
    /// 
    /// This will return an error if the body text cannot be decoded as utf-8 or
    /// if the JSON cannot be deserialized.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub async fn json<T: DeserializeOwned>(self) -> Result<T, Error> {
        serde_json::from_str::<T>(&self.text().await?).map_err(Error::from)
    }

    /// Gets the binary request
    ///
    /// This works by obtaining the response as an `ArrayBuffer`, creating a `Uint8Array` from it
    /// and then converting it to `Vec<u8>`
    /// 
    /// # Note
    /// 
    /// This consumes the request, if you need to access the body multiple times,
    /// you should `clone` the request first.
    /// 
    /// # Errors
    /// 
    /// This method may return a "RangeError"
    pub async fn binary(self) -> Result<Vec<u8>, Error> {
        let promise = self.0.array_buffer().map_err(js_to_error)?; // RangeError
        let array_buffer: ArrayBuffer = JsFuture::from(promise)
            .await
            .map_err(js_to_error)? // should never fail?
            .unchecked_into();
        let typed_buff: Uint8Array = Uint8Array::new(&array_buffer);
        let mut body = vec![0; typed_buff.length() as usize];
        typed_buff.copy_to(&mut body);
        Ok(body)
    }

    /// Executes the request, using the `fetch` API.
    pub async fn send(self) -> Result<Response, Error> {
        let request = self.0;
        let global = js_sys::global();
        let maybe_window =
            Reflect::get(&global, &JsValue::from_str("Window")).map_err(js_to_error)?;
        let promise = if !maybe_window.is_undefined() {
            let window = global.dyn_into::<web_sys::Window>().unwrap();
            window.fetch_with_request(&request)
        } else {
            let maybe_worker = Reflect::get(&global, &JsValue::from_str("WorkerGlobalScope"))
                .map_err(js_to_error)?;
            if !maybe_worker.is_undefined() {
                let worker = global.dyn_into::<web_sys::WorkerGlobalScope>().unwrap();
                worker.fetch_with_request(&request)
            } else {
                panic!("Unsupported JavaScript global context");
            }
        };

        let response = JsFuture::from(promise).await.map_err(js_to_error)?;
        response
            .dyn_into::<web_sys::Response>()
            .map_err(|e| panic!("fetch returned {:?}, not `Response` - this is a bug", e))
            .map(Response::from_raw)
    }
}

impl Clone for Request {
    fn clone(&self) -> Self {
        // Cloning the underlying request could fail if the request body has already been consumed.
        // This should not happen in normal usage since the body is consumed only when `send` is called.
        debug_assert!(!self.body_used());
        Self(self.0.clone().unwrap())
    }
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("url", &self.url())
            .field("headers", &self.headers())
            .field("body_used", &self.body_used())
            .finish()
    }
}
