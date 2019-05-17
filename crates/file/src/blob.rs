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

impl std::convert::Into<wasm_bindgen::JsValue> for BlobContents {
    fn into(self) -> wasm_bindgen::JsValue {
        self.inner
    }
}

impl std::convert::Into<BlobContents> for &str {
    fn into(self) -> BlobContents {
        BlobContents::from_raw(wasm_bindgen::JsValue::from_str(self))
    }
}

impl std::convert::Into<BlobContents> for &[u8] {
    fn into(self) -> BlobContents {
        let array = unsafe { js_sys::Uint8Array::view(self) };
        let array_clone = js_sys::Uint8Array::new(&array).buffer();
        BlobContents::from_raw(array_clone.into())
    }
}

impl std::convert::Into<BlobContents> for Blob {
    fn into(self) -> BlobContents {
        BlobContents::from_raw(self.inner.into())
    }
}

impl std::convert::Into<BlobContents> for web_sys::Blob {
    fn into(self) -> BlobContents {
        BlobContents::from_raw(self.into())
    }
}

impl std::convert::Into<BlobContents> for js_sys::ArrayBuffer {
    fn into(self) -> BlobContents {
        BlobContents::from_raw(self.into())
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
        let blob = self
            .as_raw()
            .slice_with_f64_and_f64(start as f64, end as f64)
            .unwrap_throw();

        File::new_with_options(
            self.name(),
            blob,
            None,
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
            let (millis, did_overflow_seconds) = last_modified.as_secs().overflowing_mul(1000);
            let (millis, did_overflow_millis) =
                millis.overflowing_add(last_modified.subsec_millis() as u64);
            assert!(
                !did_overflow_seconds && !did_overflow_millis,
                "last modified since epoch duration overflowed"
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
