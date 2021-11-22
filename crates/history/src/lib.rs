//! A module that provides universal session history and location information.

mod any;
mod browser;
mod error;
mod history;
mod listener;
mod location;

pub use any::{AnyHistory, AnyLocation};
pub use browser::{BrowserHistory, BrowserLocation};
pub use error::{HistoryError, HistoryResult};
pub use history::History;
pub use listener::HistoryListener;
pub use location::Location;
