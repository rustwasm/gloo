use super::{errors, ObjectStoreDuringUpgrade, ObjectStoreReadOnly, ObjectStoreReadWrite};
use js_sys::{Object, Reflect};
use once_cell::sync::Lazy;
use std::ops::Deref;
use wasm_bindgen::{prelude::*, throw_str, JsCast};
use web_sys::{IdbObjectStore, IdbTransaction};

/// An in-progress database upgrade transaction
///
/// Please do not stash the transaction. Once our code yields (e.g. over an await point that
/// doesn't involve a database method) the transaction will autocommit, and further attempts to use
/// it will return an error.
#[derive(Debug)]
pub struct TransactionDuringUpgrade {
    inner: TransactionReadWrite,
}

impl TransactionDuringUpgrade {
    pub(crate) fn new(inner: IdbTransaction) -> Self {
        Self {
            inner: TransactionReadWrite::new(inner),
        }
    }

    /// Fetch an object store
    ///
    /// Note this deliberately shadows [`TransactionReadWrite::object_store`], providing access to
    /// it through `ObjectStoreDuringUpgrade::deref`.
    pub fn object_store<'trans>(
        &'trans self,
        name: &str,
    ) -> Result<ObjectStoreDuringUpgrade, errors::ObjectStoreError> {
        object_store(self.raw(), name).map(ObjectStoreDuringUpgrade::new)
    }
}

impl Deref for TransactionDuringUpgrade {
    type Target = TransactionReadWrite;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// An in-progress database transaction
///
/// Please do not stash the transaction. Once our code yields (e.g. over an await point)
#[derive(Debug)]
pub struct TransactionReadWrite {
    inner: TransactionReadOnly,
}

impl TransactionReadWrite {
    pub(crate) fn new(inner: IdbTransaction) -> Self {
        Self {
            inner: TransactionReadOnly::new(inner),
        }
    }

    /// Fetch an object store
    ///
    /// Note this deliberately shadows [`TransactionReadOnly::object_store`], providing access to
    /// it through `Deref`.
    pub fn object_store(
        &self,
        name: &str,
    ) -> Result<ObjectStoreReadWrite, errors::ObjectStoreError> {
        object_store(self.raw(), name).map(ObjectStoreReadWrite::new)
    }
}

impl Deref for TransactionReadWrite {
    type Target = TransactionReadOnly;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// An in-progress database transaction
///
/// Please do not stash the transaction. Once our code yields (e.g. over an await point)
#[derive(Debug)]
pub struct TransactionReadOnly {
    inner: IdbTransaction,
}

impl TransactionReadOnly {
    pub(crate) fn new(inner: IdbTransaction) -> Self {
        Self { inner }
    }

    fn raw(&self) -> &IdbTransaction {
        &self.inner
    }

    /// Fetch an object store
    pub fn object_store(
        &self,
        name: &str,
    ) -> Result<ObjectStoreReadOnly, errors::ObjectStoreError> {
        object_store(self.raw(), name).map(ObjectStoreReadOnly::new)
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

impl Drop for TransactionReadOnly {
    fn drop(&mut self) {
        // indexeddb already does auto-commit. This tells it we've done so it can potentially
        // commit the transaction earlier
        // TODO if we re-enable this we need to force users to keep the transaciton alive (e.g. by
        // making object stores borrow from it).
        //self.commit();
    }
}

fn object_store(
    trans: &IdbTransaction,
    name: &str,
) -> Result<IdbObjectStore, errors::ObjectStoreError> {
    trans
        .object_store(name)
        .map_err(errors::ObjectStoreError::from)
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
