use super::{errors, ObjectStore};
use js_sys::Reflect;
use once_cell::sync::Lazy;
use std::marker::PhantomData;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::IdbTransaction;

/// A database transaction.
///
/// All interaction with a database happens in a transaction.
#[derive(Debug)]
pub struct Transaction<Ty> {
    inner: IdbTransaction,
    ty: PhantomData<Ty>,
}

impl<Ty> Transaction<Ty> {
    pub(crate) fn new(inner: IdbTransaction) -> Self {
        Self {
            inner,
            ty: PhantomData,
        }
    }

    /// Open an object store.
    pub fn object_store(&self, name: &str) -> Result<ObjectStore<Ty>, errors::ObjectStoreError> {
        self.inner
            .object_store(name)
            .map(ObjectStore::new)
            .map_err(errors::ObjectStoreError::from)
    }

    /// This function commits the transaction if supported.
    ///
    /// Any further use of the transaction (e.g. through object stores) will return errors.
    // if `IDBTransaction.prototype.commit` is present.
    pub fn commit(&self) {
        if *SUPPORTS_COMMIT {
            let t = self.inner.unchecked_ref::<SupportsCommit>();
            t.commit();
        }
    }
}

// Optional support for transaction.commit
// TODO remove logging
static SUPPORTS_COMMIT: Lazy<bool> = Lazy::new(|| {
    if let Ok(ty) = Reflect::get(&gloo_utils::window(), &"IDBTransaction".into()) {
        if let Ok(proto) = Reflect::get(&ty, &"prototype".into()) {
            if let Ok(method) = Reflect::get(&proto, &"commit".into()) {
                if !(method.is_null() || method.is_undefined()) {
                    web_sys::console::log_1(&"`IDBTransaction.prototype.commit` supported".into());
                    return true;
                }
            }
        }
    }
    web_sys::console::log_1(&"`IDBTransaction.prototype.commit` unsupported".into());
    false
});

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = ::js_sys::Object, js_name = IDBTransaction)]
    type SupportsCommit;

    // this doesn't seem to be in web_sys, perhaps because it's new.
    #[wasm_bindgen(structural, method, js_class = "IDBTransaction", js_name = commit)]
    fn commit(this: &SupportsCommit);
}
