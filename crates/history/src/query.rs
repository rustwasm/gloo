//! # Encoding and decoding strategies for query strings.
//!
//! There are various strategies to map Rust types into HTTP query strings. The [`FromQuery`] and
//! [`ToQuery`] encode the logic for how this encoding and decoding is performed. These traits
//! are public as a form of dependency inversion, so that you can override the decoding and
//! encoding strategy being used.
//!
//! These traits are used by the [`History`](crate::History) trait, which allows for modifying the
//! history state, and the [`Location`](crate::Location) struct, which allows for extracting the
//! current location (and this query).
//!
//! ## Default Strategy
//!
//! By default, any Rust type that implements [`Serialize`] or [`Deserialize`](serde::Deserialize)
//! has an implementation of [`ToQuery`] or [`FromQuery`], respectively. This implementation uses
//! the `serde_urlencoded` crate, which implements a standards-compliant `x-www-form-urlencoded`
//! encoder and decoder. Some patterns are not supported by this crate, for example it is not
//! possible to serialize arrays at the moment. If this is an issue for you, consider using the
//! `serde_qs` crate.
//!
//! Example:
//!
//! ```rust,no_run
//! use serde::{Serialize, Deserialize};
//! use gloo_history::{MemoryHistory, History};
//!
//! #[derive(Serialize)]
//! struct Query {
//!     name: String,
//! }
//!
//! let query = Query {
//!     name: "user".into(),
//! };
//!
//! let history = MemoryHistory::new();
//! history.push_with_query("index.html", &query).unwrap();
//! ```
//!
//! ## Custom Strategy
//!
//! If desired, the [`FromQuery`] and [`ToQuery`] traits can also be manually implemented on
//! types to customize the encoding and decoding strategies. See the documentation for these traits
//! for more detail on how this can be done.
use crate::error::HistoryError;
use serde::{de::DeserializeOwned, Serialize};
use std::borrow::Cow;
use std::convert::{AsRef, Infallible};

/// Type that can be encoded into a query string.
pub trait ToQuery {
    /// Error that can be returned from the conversion.
    type Error;

    /// Method to encode the query into a string.
    fn to_query(&self) -> Result<Cow<'_, str>, Self::Error>;
}

/// Type that can be decoded from a query string.
pub trait FromQuery {
    /// Target type after parsing.
    type Target;
    /// Error that can occur while parsing.
    type Error;

    /// Decode this query string into the target type.
    fn from_query(query: &str) -> Result<Self::Target, Self::Error>;
}

impl<T: Serialize> ToQuery for T {
    type Error = HistoryError;

    fn to_query(&self) -> Result<Cow<'_, str>, Self::Error> {
        serde_urlencoded::to_string(self)
            .map(Into::into)
            .map_err(Into::into)
    }
}

impl<T: DeserializeOwned> FromQuery for T {
    type Target = T;
    type Error = HistoryError;

    fn from_query(query: &str) -> Result<Self::Target, Self::Error> {
        serde_urlencoded::from_str(query).map_err(Into::into)
    }
}

/// # Encoding for raw query strings.
///
/// The [`Raw`] wrapper allows for specifying a query string directly, bypassing the encoding. If
/// you use this strategy, you need to take care to escape characters that are not allowed to
/// appear in query strings yourself.
#[derive(Debug, Clone)]
pub struct Raw<T>(pub T);

impl<T: AsRef<str>> ToQuery for Raw<T> {
    type Error = Infallible;

    fn to_query(&self) -> Result<Cow<'_, str>, Self::Error> {
        Ok(self.0.as_ref().into())
    }
}

impl<T: for<'a> From<&'a str>> FromQuery for Raw<T> {
    type Target = T;
    type Error = Infallible;

    fn from_query(query: &str) -> Result<Self::Target, Self::Error> {
        Ok(query.into())
    }
}
