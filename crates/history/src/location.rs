#[cfg(feature = "serde")]
use serde::de::DeserializeOwned;

#[cfg(feature = "serde")]
use crate::error::HistoryResult;
use crate::history::History;

/// A trait to to provide [`Location`] information.
pub trait Location: Clone + PartialEq {
    /// The [`History`] type for current [`Location`].
    type History: History<Location = Self> + 'static;

    /// Returns the `pathname` on the [`Location`] struct.
    fn path(&self) -> String;

    /// Returns the queries of current URL in [`String`]
    fn query_str(&self) -> String;

    /// Returns the queries of current URL parsed as `T`.
    #[cfg(feature = "query")]
    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned,
    {
        let query = self.query_str();
        serde_urlencoded::from_str(query.strip_prefix('?').unwrap_or("")).map_err(|e| e.into())
    }

    /// Returns the hash fragment of current URL.
    fn hash(&self) -> String;

    /// Returns the State.
    ///
    /// The implementation differs between [`Location`] type.
    ///
    /// For [`BrowserLocation`] and [`HashLocation`], state is deserialised with [`serde_wasm_bindgen`] where as
    /// [`MemoryLocation`] uses [`Any`](std::any::Any).
    #[cfg(feature = "state")]
    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static;
}
