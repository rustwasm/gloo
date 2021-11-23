//! A module that provides universal session history and location information.

#![deny(clippy::all)]
#![deny(missing_docs, missing_debug_implementations)]

mod any;
mod browser;
#[cfg(feature = "serialize")]
mod error;
mod history;
mod listener;
mod location;

pub use any::{AnyHistory, AnyLocation};
pub use browser::{BrowserHistory, BrowserLocation};

#[cfg(feature = "serialize")]
pub use error::{HistoryError, HistoryResult};
pub use history::History;
pub use listener::HistoryListener;
pub use location::Location;
