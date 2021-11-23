#[cfg(feature = "serialize")]
use serde::de::DeserializeOwned;

#[cfg(feature = "serialize")]
use crate::error::HistoryResult;
use crate::history::History;

/// A trait to to provide [`Location`] information.
pub trait Location: Clone + PartialEq {
    /// The [`History`] type for current [`Location`].
    type History: History<Location = Self> + 'static;

    /// Returns the `pathname` on the [`Location`] struct.
    fn path(&self) -> String;

    /// Returns the queries of current URL in [`String`]
    fn search(&self) -> String;

    /// Returns the queries of current URL parsed as `T`.
    #[cfg(feature = "serialize")]
    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned;

    /// Returns the hash fragment of current URL.
    fn hash(&self) -> String;

    /// Returns the State.
    ///
    /// The implementation differs between [`Location`] type.
    ///
    /// For [`BrowserLocation`] and [`HashLocation`], state is deserialised with [`serde_wasm_bindgen`] where as
    /// [`MemoryLocation`] uses [`Any`](std::any::Any).
    #[cfg(feature = "serialize")]
    fn state<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned + 'static;
}
