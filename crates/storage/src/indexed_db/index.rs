use super::{errors, util::UnreachableExt, Query, Request, Upgrade};
use serde::Deserialize;
use std::marker::PhantomData;
use wasm_bindgen::prelude::*;
use web_sys::IdbIndex;

/// An object store index
#[derive(Debug)]
pub struct Index<Ty> {
    inner: IdbIndex,
    ty: PhantomData<Ty>,
}

impl<Ty> Index<Ty> {
    pub(crate) fn new(inner: IdbIndex) -> Self {
        Self {
            inner,
            ty: PhantomData,
        }
    }

    /// Get the path to the key/keys for this index.
    pub fn key_path(&self) -> JsValue {
        self.inner.key_path().unreachable_throw()
    }

    /// How an array key is handled.
    ///
    /// If true, there is one record in the index for each item in the array. If false, then there
    /// is a single entry whose key is an array key.
    pub fn is_multi_entry(&self) -> bool {
        self.inner.multi_entry()
    }

    /// Whether keys in the index must be unique.
    pub fn is_unique(&self) -> bool {
        self.inner.unique()
    }

    /// Count the number of records in this index.
    pub async fn count(&self) -> Result<u32, errors::LifetimeError> {
        let request = self.inner.count().map_err(errors::LifetimeError::from)?;
        let count = Request::new(request, false)
            .await
            .map_err(errors::LifetimeError::from)?;
        let count = count.as_f64().unreachable_throw();
        // assume count is a valid u32
        Ok(count as u32)
    }

    /// Get the first object matching the given query in the index.
    // Strictly speaking these take a key range or undefined, but only return the first record in
    // those cases. For now I'm not implementing for those variants.
    pub async fn get_raw(&self, query: &Query) -> Result<JsValue, errors::LifetimeError> {
        let request = self
            .inner
            .get(query.as_ref())
            .map_err(errors::LifetimeError::from)?;
        Request::new(request, false)
            .await
            .map_err(errors::LifetimeError::from)
    }

    /// Get the first object matching the given query in the index and deserialize it.
    pub async fn get<V>(
        &self,
        query: &Query,
    ) -> Result<V, errors::DeSerialize<errors::LifetimeError>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let res = self
            .get_raw(query)
            .await
            .map_err(errors::DeSerialize::Other)?;
        Ok(serde_wasm_bindgen::from_value(res)?)
    }

    /// Get the objects matching the given query in the index.
    pub async fn get_all_raw(
        &self,
        query: &Query,
        count: Option<u32>,
    ) -> Result<JsValue, errors::LifetimeError> {
        let request = if let Some(count) = count {
            self.inner.get_all_with_key_and_limit(query.as_ref(), count)
        } else {
            self.inner.get_all_with_key(query.as_ref())
        }
        .map_err(errors::LifetimeError::from)?;
        Request::new(request, false)
            .await
            .map_err(errors::LifetimeError::from)
    }

    /// Get the objects matching the given query in the index.
    ///
    /// `V` should be a collection type.
    pub async fn get_all<V>(
        &self,
        query: &Query,
        count: Option<u32>,
    ) -> Result<V, errors::DeSerialize<errors::LifetimeError>>
    where
        V: for<'de> Deserialize<'de>,
    {
        let res = self
            .get_all_raw(query, count)
            .await
            .map_err(errors::DeSerialize::Other)?;
        Ok(serde_wasm_bindgen::from_value(res)?)
    }
}

impl Index<Upgrade> {
    /// Change the name of this index.
    ///
    /// # Panics
    ///
    /// Currently this function panics rather than returning an error if the underlying js fn
    /// throws. This is a limitation of `web_sys`.
    pub fn set_name(&self, name: &str) -> Result<(), errors::CreateIndexError> {
        self.inner.set_name(name);
        Ok(())
    }
}
