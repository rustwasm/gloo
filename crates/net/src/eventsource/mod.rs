//! Wrapper around the `EventSource` API
//!
//! This API is provided in the following flavors:
//! - [Futures API][futures]

pub mod futures;

use std::fmt;

/// The state of the EventSource.
///
/// See [`EventSource.readyState` on MDN](https://developer.mozilla.org/en-US/docs/Web/API/EventSource/readyState)
/// to learn more.
#[derive(Copy, Clone, Debug)]
pub enum State {
    /// The connection has not yet been established.
    Connecting,
    /// The EventSource connection is established and communication is possible.
    Open,
    /// The connection has been closed or could not be opened.
    Closed,
}

/// Error returned by the EventSource
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
#[allow(missing_copy_implementations)]
pub enum EventSourceError {
    /// The `error` event
    ConnectionError,
}

impl fmt::Display for EventSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventSourceError::ConnectionError => write!(f, "EventSource connection failed"),
        }
    }
}
