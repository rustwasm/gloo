use crate::{Blob, File};
use std::{ops::Deref, rc::Rc};

use wasm_bindgen::UnwrapThrowExt;
use web_sys::Url;

struct ObjectUrlAllocation {
    url: String,
}

impl Drop for ObjectUrlAllocation {
    fn drop(&mut self) {
        web_sys::Url::revoke_object_url(&self.url).unwrap_throw();
    }
}

/// A resource wrapper around [`URL.createObjectURL`] / [`URL.revokeObjectURL`].
///
/// A [`Blob`], in particular a [`File`], can be converted to a short URL representing its data with the above methods.
/// An [`ObjectUrl`] can be cheaply cloned and shared and revokes the underlying URL when the last reference is dropped.
///
/// Note that multiple urls can be created for the same blob, without being guaranteed to be de-deduplicated.
///
/// # Example
///
/// ```rust,no_run
/// use gloo_file::{Blob, ObjectUrl};
///
/// let blob = Blob::new("hello world");
/// let object_url = ObjectUrl::from(blob);
/// ```
///
/// [`URL.createObjectURL`]: https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL
/// [`URL.revokeObjectURL`]: https://developer.mozilla.org/en-US/docs/Web/API/URL/revokeObjectURL
/// [`File`]: crate::File
#[derive(Clone)]
pub struct ObjectUrl {
    inner: Rc<ObjectUrlAllocation>,
}

impl From<File> for ObjectUrl {
    fn from(file: File) -> Self {
        Blob::from(file).into()
    }
}

impl From<Blob> for ObjectUrl {
    fn from(blob: Blob) -> Self {
        web_sys::Blob::from(blob).into()
    }
}

impl From<web_sys::Blob> for ObjectUrl {
    fn from(blob: web_sys::Blob) -> Self {
        let url = Url::create_object_url_with_blob(&blob).unwrap_throw();
        let inner = Rc::new(ObjectUrlAllocation { url });
        Self { inner }
    }
}

// Note: some browsers support Url::create_object_url_with_source but this is deprecated!
// https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL#using_object_urls_for_media_streams

impl Deref for ObjectUrl {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner.url
    }
}
