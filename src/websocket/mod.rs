//! Wrapper around `WebSocket` API
//!
//! This API is provided in the following flavors:
//! - [Futures API][futures]

pub mod events;
pub mod futures;

use events::{CloseEvent, ErrorEvent};
use gloo_utils::errors::JsError;
use std::fmt;

/// Message sent to and received from WebSocket.
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Copy, Clone, Debug)]
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

/// Error returned by WebSocket
#[derive(Debug)]
#[non_exhaustive]
pub enum WebSocketError {
    /// The `error` event
    ConnectionError(ErrorEvent),
    /// The `close` event
    ConnectionClose(CloseEvent),
    /// Message failed to send.
    MessageSendError(JsError),
}

impl fmt::Display for WebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WebSocketError::ConnectionError(e) => write!(f, "{}", e.message),
            WebSocketError::ConnectionClose(e) => write!(
                f,
                "WebSocket Closed: code: {}, reason: {}",
                e.code, e.reason
            ),
            WebSocketError::MessageSendError(e) => write!(f, "{}", e),
        }
    }
}
