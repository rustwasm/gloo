//! A module that provides universal session history and location information.

#![deny(clippy::all)]
#![deny(missing_docs, missing_debug_implementations)]

mod any;
mod browser;
#[cfg(feature = "query")]
mod error;
mod hash;
mod history;
mod listener;
mod location;
mod memory;
#[cfg(feature = "query")]
pub mod query;
mod state;
mod utils;

pub use any::AnyHistory;
pub use browser::BrowserHistory;
pub use hash::HashHistory;
pub use memory::MemoryHistory;

#[cfg(feature = "query")]
pub use error::{HistoryError, HistoryResult};
pub use history::History;
pub use listener::HistoryListener;
pub use location::Location;
