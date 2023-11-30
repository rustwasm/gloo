//! A module that provides universal session history and location information.

#![deny(clippy::all)]
#![deny(missing_docs, missing_debug_implementations)]

mod any;
#[cfg(not(target_os = "wasi"))]
mod browser;
#[cfg(feature = "query")]
mod error;
#[cfg(not(target_os = "wasi"))]
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
#[cfg(not(target_os = "wasi"))]
pub use browser::BrowserHistory;
#[cfg(not(target_os = "wasi"))]
pub use hash::HashHistory;
pub use memory::MemoryHistory;

#[cfg(feature = "query")]
pub use error::{HistoryError, HistoryResult};
pub use history::History;
pub use listener::HistoryListener;
pub use location::Location;
