use crate::Sealed;
use std::{
    ops::Deref,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use wasm_bindgen::{prelude::*, throw_str, JsCast};

/// This trait is used to overload the `Blob::new_with_options` function, allowing a variety of
/// types to be used to create a `Blob`. Ignore this, and use &\[u8], &str, etc to create a `Blob`.
///
/// The trait is sealed: it can only be implemented by types in this
/// crate, as this crate relies on invariants regarding the `JsValue` returned from `into_jsvalue`.
pub trait BlobContents: Sealed {
    /// # Safety
    ///
    /// For `&[u8]` and `&str`, the returned `Uint8Array` must be modified,
    /// and must not be kept past the lifetime of the original slice.
    unsafe fn into_jsvalue(self) -> JsValue;
}

impl<'a> Sealed for &'a str {}
impl<'a> BlobContents for &'a str {
    unsafe fn into_jsvalue(self) -> JsValue {
        // Converting a Rust string to a JS string re-encodes it from UTF-8 to UTF-16,
        // and `Blob` re-encodes JS strings from UTF-16 to UTF-8.
        // So, it's better to just pass the original bytes of the Rust string to `Blob`
        // and avoid the round trip through UTF-16.
        self.as_bytes().into_jsvalue()
    }
}

impl<'a> Sealed for &'a [u8] {}
impl<'a> BlobContents for &'a [u8] {
    unsafe fn into_jsvalue(self) -> JsValue {
        js_sys::Uint8Array::view(self).into()
    }
}

impl Sealed for js_sys::ArrayBuffer {}
impl BlobContents for js_sys::ArrayBuffer {
    unsafe fn into_jsvalue(self) -> JsValue {
        self.into()
    }
}

impl Sealed for js_sys::JsString {}
impl BlobContents for js_sys::JsString {
    unsafe fn into_jsvalue(self) -> JsValue {
        self.into()
    }
}

impl Sealed for Blob {}
impl BlobContents for Blob {
    unsafe fn into_jsvalue(self) -> JsValue {
        self.into()
    }
}

/// A [`Blob`](https://developer.mozilla.org/en-US/docs/Web/API/Blob).
///
/// `Blob`s can be created directly from `&str`, `&[u8]`, and `js_sys::ArrayBuffer`s using the
/// `Blob::new` or `Blob::new_with_options` functions.
#[derive(Debug, Clone, PartialEq)]
pub struct Blob {
    inner: web_sys::Blob,
}

impl Blob {
    /// Create a new `Blob` from a `&str`, `&[u8]` or `js_sys::ArrayBuffer`.
    pub fn new<T>(content: T) -> Blob
    where
        T: BlobContents,
    {
        Blob::new_with_options(content, None)
    }

    /// Like `new`, but allows specifying the MIME type (also known as *content type* or *media
    /// type*) of the `Blob`.
    pub fn new_with_options<T>(content: T, mime_type: Option<&str>) -> Blob
    where
        T: BlobContents,
    {
        let mut properties = web_sys::BlobPropertyBag::new();
        if let Some(mime_type) = mime_type {
            properties.type_(mime_type);
        }

        // SAFETY: The slice will live for the duration of this function call,
        // and `new Blob()` will not modify the bytes or keep a reference to them past the end of the call.
        let parts = js_sys::Array::of1(&unsafe { content.into_jsvalue() });
        let inner = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &properties);

        Blob::from(inner.unwrap_throw())
    }

    pub fn slice(&self, start: u64, end: u64) -> Self {
        let start = safe_u64_to_f64(start);
        let end = safe_u64_to_f64(end);

        let b: &web_sys::Blob = self.as_ref();
        Blob::from(b.slice_with_f64_and_f64(start, end).unwrap_throw())
    }

    /// The number of bytes in the Blob/File.
    pub fn size(&self) -> u64 {
        safe_f64_to_u64(self.inner.size())
    }

    /// The statically typed MIME type (also known as *content type* or *media type*) of the `File`
    /// or `Blob`.
    #[cfg(feature = "mime")]
    pub fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> {
        self.raw_mime_type().parse()
    }

    /// The raw MIME type (also known as *content type* or *media type*) of the `File` or
    /// `Blob`.
    pub fn raw_mime_type(&self) -> String {
        self.inner.type_()
    }
}

impl From<web_sys::Blob> for Blob {
    fn from(blob: web_sys::Blob) -> Self {
        Blob { inner: blob }
    }
}

impl From<web_sys::File> for Blob {
    fn from(file: web_sys::File) -> Self {
        Blob { inner: file.into() }
    }
}

impl From<Blob> for web_sys::Blob {
    fn from(blob: Blob) -> Self {
        blob.inner
    }
}

impl From<Blob> for JsValue {
    fn from(blob: Blob) -> Self {
        blob.inner.into()
    }
}

impl AsRef<web_sys::Blob> for Blob {
    fn as_ref(&self) -> &web_sys::Blob {
        self.inner.as_ref()
    }
}

impl AsRef<JsValue> for Blob {
    fn as_ref(&self) -> &JsValue {
        self.inner.as_ref()
    }
}

/// A [`File`](https://developer.mozilla.org/en-US/docs/Web/API/File).
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    // the trick here is that we know the contents of `inner` are a file, even though that type
    // information is not stored. It is the same trick as is used in `web_sys`.
    inner: Blob,
}

impl File {
    /// Create a new `File` with the given name and contents.
    ///
    /// `contents` can be `&str`, `&[u8]`, or `js_sys::ArrayBuffer`.
    pub fn new<T>(name: &str, contents: T) -> File
    where
        T: BlobContents,
    {
        Self::new_with_options(name, contents, None, None)
    }

    /// Like `File::new`, but allows customizing the MIME type (also
    /// known as *content type* or *media type*), and the last modified time.
    ///
    /// `std::time::SystemTime` is a low level type, use a crate like
    /// [`chrono`](https://docs.rs/chrono/0.4.10/chrono/) to work with a more user-friendly
    /// representation of time.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use chrono::prelude::*;
    /// use gloo_file::File;
    ///
    /// // Just create a dummy `gloo::file::File` for demonstration purposes.
    /// let example_file = File::new_with_options(
    ///     "motivation.txt",
    ///     "live your best life",
    ///     Some("text/plain"),
    ///     Some(Utc::now().into())
    /// );
    /// assert_eq!(example_file.name(), String::from("motivation.txt"));
    /// assert_eq!(example_file.raw_mime_type(), String::from("text/plain"));
    /// ```
    pub fn new_with_options<T>(
        name: &str,
        contents: T,
        mime_type: Option<&str>,
        last_modified_time: Option<SystemTime>,
    ) -> File
    where
        T: BlobContents,
    {
        let mut options = web_sys::FilePropertyBag::new();
        if let Some(mime_type) = mime_type {
            options.type_(mime_type);
        }

        if let Some(last_modified_time) = last_modified_time {
            let duration = match last_modified_time.duration_since(UNIX_EPOCH) {
                Ok(duration) => safe_u128_to_f64(duration.as_millis()),
                Err(time_err) => -safe_u128_to_f64(time_err.duration().as_millis()),
            };
            options.last_modified(duration);
        }

        // SAFETY: The original reference will live for the duration of this function call,
        // and `new File()` won't mutate the `Uint8Array` or keep a reference to it past the end of this call.
        let parts = js_sys::Array::of1(&unsafe { contents.into_jsvalue() });
        let inner = web_sys::File::new_with_u8_array_sequence_and_options(&parts, name, &options)
            .unwrap_throw();

        File::from(inner)
    }

    /// Gets the file name.
    pub fn name(&self) -> String {
        let f: &web_sys::File = self.as_ref();
        f.name()
    }

    /// Gets the time that the file was last modified.
    ///
    /// `std::time::SystemTime` is a low level type, use a crate like
    /// [`chrono`](https://docs.rs/chrono/0.4.10/chrono/) to work with more user-friendly
    /// representations of time. For example:
    ///
    /// ```rust,no_run
    /// use chrono::prelude::*;
    /// use gloo_file::File;
    ///
    /// // Just create a dummy `gloo::file::File` for demonstration purposes.
    /// let example_file = File::new("test_file.txt", "<almost empty contents>");
    /// let date: DateTime<Utc> = example_file.last_modified_time().into();
    /// ```
    pub fn last_modified_time(&self) -> SystemTime {
        let f: &web_sys::File = self.as_ref();
        match f.last_modified() {
            pos if pos >= 0.0 => UNIX_EPOCH + Duration::from_millis(safe_f64_to_u64(pos)),
            neg => UNIX_EPOCH - Duration::from_millis(safe_f64_to_u64(-neg)),
        }
    }

    /// Create a new `File` from a sub-part of this `File`.
    pub fn slice(&self, start: u64, end: u64) -> Self {
        let blob = self.deref().slice(start, end);

        let raw_mime_type = self.raw_mime_type();
        let mime_type = if raw_mime_type.is_empty() {
            None
        } else {
            Some(raw_mime_type)
        };

        File::new_with_options(
            &self.name(),
            blob,
            mime_type.as_deref(),
            Some(self.last_modified_time()),
        )
    }
}

impl From<web_sys::File> for File {
    fn from(file: web_sys::File) -> Self {
        File {
            inner: Blob::from(web_sys::Blob::from(file)),
        }
    }
}

impl Deref for File {
    type Target = Blob;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<web_sys::File> for File {
    fn as_ref(&self) -> &web_sys::File {
        <Blob as AsRef<web_sys::Blob>>::as_ref(&self.inner).unchecked_ref()
    }
}

impl AsRef<web_sys::Blob> for File {
    fn as_ref(&self) -> &web_sys::Blob {
        self.inner.as_ref()
    }
}

impl From<File> for Blob {
    fn from(file: File) -> Self {
        file.inner
    }
}

// utility methods
// ===============

/// JavaScript only has `f64`, which has a maximum accurate integer size of`2^53 - 1`. So we use
/// this to safely convert from larger integers to `f64`.  See
/// [Number.MAX_SAFE_INTEGER](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER)
fn safe_u64_to_f64(number: u64) -> f64 {
    // Max integer stably representable by f64
    if number > (js_sys::Number::MAX_SAFE_INTEGER as u64) {
        throw_str("a rust number was too large and could not be represented in JavaScript");
    }
    number as f64
}

fn safe_u128_to_f64(number: u128) -> f64 {
    // Max integer stably representable by f64
    const MAX_SAFE_INTEGER: u128 = js_sys::Number::MAX_SAFE_INTEGER as u128; // (2^53 - 1)
    if number > MAX_SAFE_INTEGER {
        throw_str("a rust number was too large and could not be represented in JavaScript");
    }
    number as f64
}

/// Like safe_u64_to_f64, but additionally checks that the number is an integer.
fn safe_f64_to_u64(number: f64) -> u64 {
    // Max integer stably representable by f64
    if number > js_sys::Number::MAX_SAFE_INTEGER {
        throw_str("a rust number was too large and could not be represented in JavaScript");
    }

    if number.fract() != 0.0 {
        throw_str(
            "a number could not be converted to an integer because it was not a whole number",
        );
    }
    number as u64
}
