use std::time::Duration;

use wasm_bindgen::UnwrapThrowExt;

pub trait BlobLike {
    fn size(&self) -> u64 {
        self.as_raw().size() as u64
    }

    #[cfg(feature = "mime")]
    fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> {
        self.raw_mime_type().parse()
    }

    fn raw_mime_type(&self) -> String {
        self.as_raw().type_()
    }

    fn as_raw(&self) -> &web_sys::Blob;

    fn slice(&self, start: u64, end: u64) -> Self;
}

#[derive(Debug, Clone)]
pub struct Blob {
    inner: web_sys::Blob,
}

#[derive(Debug, Clone)]
pub struct BlobContents {
    inner: wasm_bindgen::JsValue,
}
impl BlobContents {
    fn from_raw(inner: wasm_bindgen::JsValue) -> BlobContents {
        BlobContents { inner }
    }
}

impl std::convert::From<BlobContents> for wasm_bindgen::JsValue {
    fn from(blob_contents: BlobContents) -> wasm_bindgen::JsValue {
        blob_contents.inner
    }
}

impl std::convert::From<&str> for BlobContents {
    fn from(str: &str) -> Self {
        BlobContents::from_raw(wasm_bindgen::JsValue::from_str(str))
    }
}

impl std::convert::From<&[u8]> for BlobContents {
    fn from(buffer: &[u8]) -> Self {
        let array = unsafe { js_sys::Uint8Array::view(buffer) };
        BlobContents::from_raw(js_sys::Uint8Array::new(&array).into())
    }
}

impl std::convert::From<Blob> for BlobContents {
    fn from(blob: Blob) -> Self {
        BlobContents::from_raw(blob.inner.into())
    }
}

impl std::convert::From<File> for BlobContents {
    fn from(file: File) -> Self {
        BlobContents::from_raw(file.inner.into())
    }
}

impl std::convert::From<web_sys::Blob> for BlobContents {
    fn from(blob: web_sys::Blob) -> Self {
        BlobContents::from_raw(blob.into())
    }
}

impl std::convert::From<js_sys::ArrayBuffer> for BlobContents {
    fn from(buffer: js_sys::ArrayBuffer) -> Self {
        BlobContents::from_raw(buffer.into())
    }
}

impl Blob {
    pub fn new<T>(content: T) -> Blob
    where
        T: std::convert::Into<BlobContents>,
    {
        let parts = js_sys::Array::of1(&content.into().inner);
        let inner = web_sys::Blob::new_with_u8_array_sequence(&parts);
        Blob::from_raw(inner.unwrap_throw())
    }

    pub fn new_with_options<T>(content: T, mime_type: String) -> Blob
    where
        T: std::convert::Into<BlobContents>,
    {
        let mut properties = web_sys::BlobPropertyBag::new();
        properties.type_(&mime_type);

        let parts = js_sys::Array::of1(&content.into().inner);
        let inner = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &properties);

        Blob::from_raw(inner.unwrap_throw())
    }

    pub fn from_raw(inner: web_sys::Blob) -> Blob {
        Blob { inner }
    }
}

impl BlobLike for Blob {
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }

    fn slice(&self, start: u64, end: u64) -> Self {
        ensure_representable(start);
        ensure_representable(end);
        Blob::from_raw(
            self.as_raw()
                .slice_with_f64_and_f64(start as f64, end as f64)
                .unwrap_throw(),
        )
    }
}
impl BlobLike for File {
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }

    fn slice(&self, start: u64, end: u64) -> Self {
        ensure_representable(start);
        ensure_representable(end);
        let blob = self
            .as_raw()
            .slice_with_f64_and_f64(start as f64, end as f64)
            .unwrap_throw();
        let raw_mime_type = self.raw_mime_type();
        let mime_type = if raw_mime_type == "" {
            None
        } else {
            Some(raw_mime_type)
        };

        File::new_with_options(
            self.name(),
            blob,
            mime_type,
            Some(self.last_modified_since_epoch()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub(crate) inner: web_sys::File,
}

impl File {
    pub fn new<T>(name: String, contents: T) -> File
    where
        T: std::convert::Into<BlobContents>,
    {
        let parts = js_sys::Array::of1(&contents.into().inner);
        let inner = web_sys::File::new_with_u8_array_sequence(&parts, &name).unwrap_throw();

        File::from_raw(inner)
    }

    pub fn new_with_options<T>(
        name: String,
        contents: T,
        mime_type: Option<String>,
        last_modified_since_epoch: Option<Duration>,
    ) -> File
    where
        T: std::convert::Into<BlobContents>,
    {
        let mut options = web_sys::FilePropertyBag::new();
        if let Some(mime_type) = mime_type {
            options.type_(&mime_type);
        }

        if let Some(last_modified) = last_modified_since_epoch {
            // Max integer stably representable by f64
            let max_integer: u128 = (2.0f64).powi(54) as u128;
            let millis = last_modified.as_millis();
            assert!(
                millis <= max_integer,
                "timestamp is too large and cannot be represented in JavaScript"
            );
            options.last_modified(millis as f64);
        }
        let parts = js_sys::Array::of1(&contents.into().inner);
        let inner = web_sys::File::new_with_u8_array_sequence_and_options(&parts, &name, &options)
            .unwrap_throw();

        File::from_raw(inner)
    }

    pub fn from_raw(inner: web_sys::File) -> File {
        File { inner }
    }

    pub fn name(&self) -> String {
        self.inner.name()
    }

    pub fn last_modified_since_epoch(&self) -> Duration {
        Duration::from_millis(self.inner.last_modified() as u64)
    }

    pub fn size(&self) -> u64 {
        self.inner.size() as u64
    }
}

fn ensure_representable(number: u64) {
    // Max integer stably representable by f64
    let max_integer: u64 = (2.0f64).powi(54) as u64;
    assert!(
        number <= max_integer,
        format!(
            "{} is too large and cannot be represented in JavaScript",
            number
        )
    );
}