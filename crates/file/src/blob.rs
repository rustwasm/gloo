use wasm_bindgen::UnwrapThrowExt;

pub trait BlobLike {
    fn size(&self) -> usize {
        self.as_raw().size() as usize
    }

    #[cfg(feature = "mime")]
    fn mime_type(&self) -> Result<mime::Mime, mime::FromStrError> {
        self.raw_mime_type().parse()
    }

    fn raw_mime_type(&self) -> String {
        self.as_raw().type_()
    }

    fn as_raw(&self) -> &web_sys::Blob;
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

impl std::convert::Into<BlobContents> for js_sys::ArrayBuffer {
    fn into(self) -> BlobContents {
        BlobContents::from_raw(self.into())
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlobBuilder {
    contents: Option<BlobContents>,
    mime_type: Option<String>,
}

impl BlobBuilder {
    pub fn new() -> BlobBuilder {
        Default::default()
    }

    pub fn contents(self, contents: impl std::convert::Into<BlobContents>) -> BlobBuilder {
        let contents = Some(contents.into());
        BlobBuilder {
            contents,
            mime_type: self.mime_type,
        }
    }

    pub fn mime_type(self, mime_type: String) -> BlobBuilder {
        BlobBuilder {
            contents: self.contents,
            mime_type: Some(mime_type),
        }
    }

    pub fn build(self) -> Blob {
        Blob::new(self.contents, self.mime_type)
    }
}

impl Blob {
    pub fn new<T>(content: Option<T>, mime_type: Option<String>) -> Blob
    where
        T: std::convert::Into<BlobContents>,
    {
        let mut properties = web_sys::BlobPropertyBag::new();
        if let Some(mime_type) = mime_type {
            properties.type_(&mime_type);
        }

        let inner = if let Some(content) = content {
            let parts = js_sys::Array::of1(&content.into().inner);
            web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &properties)
        } else {
            web_sys::Blob::new()
        };

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
}
impl BlobLike for File {
    fn as_raw(&self) -> &web_sys::Blob {
        &self.inner
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub(crate) inner: web_sys::File,
}

#[derive(Debug, Clone)]
pub struct FileBuilder {
    name: String,
    contents: Option<BlobContents>,
    mime_type: Option<String>,
    last_modified_date: Option<u64>,
}

impl FileBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            contents: None,
            mime_type: None,
            last_modified_date: None,
        }
    }

    pub fn contents<T>(self, contents: T) -> Self
    where
        T: std::convert::Into<BlobContents>,
    {
        Self {
            name: self.name,
            contents: Some(contents.into()),
            mime_type: self.mime_type,
            last_modified_date: self.last_modified_date,
        }
    }

    pub fn name(self, name: String) -> Self {
        Self {
            name,
            contents: self.contents,
            mime_type: self.mime_type,
            last_modified_date: self.last_modified_date,
        }
    }

    pub fn mime_type(self, mime_type: String) -> Self {
        Self {
            name: self.name,
            contents: self.contents,
            mime_type: Some(mime_type),
            last_modified_date: self.last_modified_date,
        }
    }

    pub fn last_modified_date(self, last_modified_date: u64) -> Self {
        Self {
            name: self.name,
            contents: self.contents,
            mime_type: self.mime_type,
            last_modified_date: Some(last_modified_date),
        }
    }

    pub fn build(self) -> File {
        File::new(
            self.name,
            self.contents,
            self.mime_type,
            self.last_modified_date,
        )
    }
}

impl File {
    pub fn new<T>(
        name: String,
        contents: Option<T>,
        mime_type: Option<String>,
        last_modified_date: Option<u64>,
    ) -> File
    where
        T: std::convert::Into<BlobContents>,
    {
        let mut options = web_sys::FilePropertyBag::new();
        if let Some(mime_type) = mime_type {
            options.type_(&mime_type);
        }
        if let Some(last_modified_date) = last_modified_date {
            options.last_modified(last_modified_date as f64);
        }
        let parts = contents
            .map(|c| js_sys::Array::of1(&c.into().inner))
            .unwrap_or_else(js_sys::Array::new);
        let inner = web_sys::File::new_with_u8_array_sequence_and_options(&parts, &name, &options)
            .unwrap_throw();

        File::from_raw(inner)
    }

    pub fn from_raw(inner: web_sys::File) -> File {
        File { inner }
    }

    pub fn as_raw(&self) -> &web_sys::File {
        &self.inner
    }

    pub fn name(&self) -> String {
        self.inner.name()
    }

    pub fn last_modified_date(&self) -> u64 {
        self.inner.last_modified() as u64
    }

    pub fn size(&self) -> u64 {
        self.inner.size() as u64
    }
}
