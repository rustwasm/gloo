use crate::{Blob, Sealed};
use std::{ops::Deref, rc::Rc};

use wasm_bindgen::UnwrapThrowExt;
use web_sys::Url;

struct ObjectUrlAllocation {
    url: String,
}

/// This trait is used to overload the `Url::create_object_url` function, allowing a variety of
/// types to be used to create a [`ObjectUrl`]. Ignore this, and use [`Blob`] or [`File`] to create one.
///
/// The trait is sealed: it can only be implemented by types in this
/// crate, as this crate relies on invariants regarding the `JsValue` returned from `into_jsvalue`.
///
/// [`File`]: crate::File
pub trait ObjectUrlTarget: Sealed {
    fn create_new_object_url(&self) -> String;
}

impl ObjectUrlTarget for Blob {
    fn create_new_object_url(&self) -> String {
        Url::create_object_url_with_blob(self.as_ref()).unwrap_throw()
    }
}

impl Sealed for web_sys::Blob {}
impl ObjectUrlTarget for web_sys::Blob {
    fn create_new_object_url(&self) -> String {
        Url::create_object_url_with_blob(self).unwrap_throw()
    }
}

// Note: some browsers support Url::create_object_url_with_source but this is deprecated!
// https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL#using_object_urls_for_media_streams

impl ObjectUrlAllocation {
    fn allocate_new_object_url(target: &dyn ObjectUrlTarget) -> Self {
        let url = target.create_new_object_url();
        Self { url }
    }
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
/// [`URL.createObjectURL`]: https://developer.mozilla.org/en-US/docs/Web/API/URL/createObjectURL
/// [`URL.revokeObjectURL`]: https://developer.mozilla.org/en-US/docs/Web/API/URL/revokeObjectURL
/// [`File`]: crate::File
#[derive(Clone)]
pub struct ObjectUrl {
    inner: Rc<ObjectUrlAllocation>,
}

impl ObjectUrl {
    /// Create a new ObjectUrl from a [`Blob`] or [`File`].
    ///
    /// [`File`]: crate::File
    pub fn new(target: &dyn ObjectUrlTarget) -> Self {
        Self {
            inner: Rc::new(ObjectUrlAllocation::allocate_new_object_url(target)),
        }
    }
}

impl Deref for ObjectUrl {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner.url
    }
}
