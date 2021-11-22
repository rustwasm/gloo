use serde::de::DeserializeOwned;

use crate::error::HistoryResult;
use crate::history::History;

/// A trait to to provide [`Location`] information.
pub trait Location: Clone + PartialEq {
    type History: History<Location = Self> + 'static;

    /// Returns the `pathname` on the [`Location`] struct.
    fn pathname(&self) -> String;

    /// Returns the queries of current URL in [`String`]
    fn search(&self) -> String;

    /// Returns the queries of current URL parsed as `T`.
    fn query<T>(&self) -> HistoryResult<T>
    where
        T: DeserializeOwned;

    /// Returns the hash fragment of current URL.
    fn hash(&self) -> String;
}
