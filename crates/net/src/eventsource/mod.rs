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
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    /// The connection has not yet been established.
    Connecting,
    /// The EventSource connection is established and communication is possible.
    Open,
    /// The connection has been closed or could not be opened.
    Closed,
}

impl State {
    /// Returns the state as a &'static str.
    ///
    /// # Example
    ///
    /// ```
    /// # use gloo_net::eventsource::State;
    /// assert_eq!(State::Connecting.as_str(), "connecting");
    /// assert_eq!(State::Open.as_str(), "open");
    /// assert_eq!(State::Closed.as_str(), "closed");
    /// ```
    pub const fn as_str(&self) -> &'static str {
        match self {
            State::Connecting => "connecting",
            State::Open => "open",
            State::Closed => "closed",
        }
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<u16> for State {
    fn from(state: u16) -> Self {
        match state {
            0 => State::Connecting,
            1 => State::Open,
            2 => State::Closed,
            _ => unreachable!("Invalid readyState"),
        }
    }
}

impl From<State> for u16 {
    fn from(state: State) -> Self {
        match state {
            State::Connecting => 0,
            State::Open => 1,
            State::Closed => 2,
        }
    }
}

/// Error returned by the EventSource
#[derive(Debug, Clone, Eq, PartialEq)]
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
