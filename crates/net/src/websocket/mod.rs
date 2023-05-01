//! Wrapper around `WebSocket` API
//!
//! This API is provided in the following flavors:
//! - [Futures API][futures]

pub mod events;
pub mod futures;

use events::CloseEvent;
use gloo_utils::errors::JsError;
use std::fmt;

/// Message sent to and received from WebSocket.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Message {
    /// String message
    Text(String),
    /// ArrayBuffer parsed into bytes
    Bytes(Vec<u8>),
}

/// The state of the websocket.
///
/// See [`WebSocket.readyState` on MDN](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState)
/// to learn more.
// This trait implements `Ord`, use caution when changing the order of the variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    /// The connection has not yet been established.
    Connecting,
    /// The WebSocket connection is established and communication is possible.
    Open,
    /// The connection is going through the closing handshake, or the close() method has been
    /// invoked.
    Closing,
    /// The connection has been closed or could not be opened.
    Closed,
}

impl From<u16> for State {
    fn from(state: u16) -> Self {
        match state {
            0 => State::Connecting,
            1 => State::Open,
            2 => State::Closing,
            3 => State::Closed,
            _ => unreachable!("invalid state"),
        }
    }
}

impl From<State> for u16 {
    fn from(state: State) -> Self {
        match state {
            State::Connecting => 0,
            State::Open => 1,
            State::Closing => 2,
            State::Closed => 3,
        }
    }
}

/// Error returned by WebSocket
#[derive(Debug)]
#[non_exhaustive]
pub enum WebSocketError {
    /// The `error` event
    ConnectionError,
    /// The `close` event
    ConnectionClose(CloseEvent),
    /// Message failed to send.
    MessageSendError(JsError),
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketError::ConnectionError => write!(f, "WebSocket connection failed"),
            WebSocketError::ConnectionClose(e) => write!(
                f,
                "WebSocket Closed: code: {}, reason: {}",
                e.code, e.reason
            ),
            WebSocketError::MessageSendError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for WebSocketError {}

#[cfg(test)]
mod tests {
    use crate::is_strictly_sorted;

    use super::*;

    #[test]
    fn test_order() {
        let expected_order = vec![
            State::Connecting,
            State::Open,
            State::Closing,
            State::Closed,
        ];

        assert!(is_strictly_sorted(&expected_order));

        // Check that the u16 conversion is also sorted
        let order: Vec<_> = expected_order.iter().map(|s| u16::from(*s)).collect();
        assert!(is_strictly_sorted(&order));
    }
}
