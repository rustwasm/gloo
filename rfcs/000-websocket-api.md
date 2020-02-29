## Summary

Add a low-level callbacks library and a mid-level futures library for websockets.

 - The [HTML standard]
 - The [protocol specification]

## Motivation

Websockets allow the browser (or other user agent) and the server to communicate asynchronously
over a TCP socket. Along with conventional asynchronous requests (`fetch` or `XmlHttpRequest`),
they provide the ability for the website to interact with the internet, and have become a vital
part of modern application development.

Despite their importance, the API websockets provide can be fiddly to use. All information from the
socket is received via. callbacks, and the various methods can throw exceptions if used
incorrectly. Moreover, very little information is provided on error, as otherwise websockets
could be used to [probe the user agent's network environment, in preparation for an attack][failure-information].

This RFC proposes to make websockets easier to use from rust. Firstly, it will map the raw API
closely, providing a "rust-y" version of callbacks and error management. Then, it will provide
a futures-based library, making it more convenient to handle the sending and receiving of
messages.

## Detailed Explanation

So as to be consistent with the rest of `gloo` this library will be implemented in 2 distinct
modules: `callback` and `futures`. The `callback` API will be a thin wrapper around `web-sys`,
providing some type safety and reducing boilerplate where possible. The `futures` API will use
rust async concepts (`Stream + Sync`) to make it as easy and risk-free to use as possible.

### `callback` api

```rust
pub struct WebSocket { .. }

impl WebSocket {
    /// Connect to a websocket with remote at `url`.
    ///
    ///  - The open listener will run once the connection is ready to receive send requests.
    ///  - The message listener will run when there is a message received from the socket.
    ///  - The close listener will fire when the closing handshake is started.
    ///  - The error listener will fire when there is an error with the websocket. This can be
    ///    caused by (amongst other reasons)
    ///    - Trying to send data when the websocket buffer is full.
    ///    - The websocket could not be established.
    ///    - Trying to close a websocket before it is established.
    ///    - The page is closed before the websocket is established.
    ///
    ///    Note that the error event receives no information about the error. This is
    ///    intentional to prevent network sniffing. See
    ///    [the spec](https://html.spec.whatwg.org/multipage/web-sockets.html#closeWebSocket)
    ///    for more details.
    pub fn connect(
        url: &str,
        // Note this function cannot take `Option`s as it would not be able to resolve types in the
        // `None` case. Would have to be `Option<Box<dyn FnOnce + 'static>>`.
        on_open: impl FnOnce() + 'static,
        mut on_error: impl FnMut() + 'static,
        on_close: impl FnOnce(CloseEvent) + 'static,
        mut on_message: impl FnMut(Message) + 'static,
    ) -> Result<Self, ConnectError>
    { .. }

    /// Send a text message over the socket.
    pub fn send_text(&self, data: &str) -> Result<(), SendError> { .. }

    /// Send a binary message over the socket.
    pub fn send_binary(&self, data: &[u8]) -> Result<(), SendError> { .. }

    /// Whether it is possible to send messages.
    pub fn can_send(&self) -> bool { .. }

    /// Get the current state of the websocket.
    pub fn state(&self) -> State { .. }

    /// The amount of data that has been sent using the `WebSocket::send` function, but that is
    /// still waiting to be sent over the TCP connection.
    pub fn buffered_amount(&self) -> u32 { .. }

    /// The extensions in use.
    ///
    /// This is `None` before the connection is established.
    pub fn extensions(&self) -> Option<String> { .. }

    /// The subprotocol in use.
    ///
    /// This is `None` before the connection is established.
    pub fn protocol(&self) -> String { .. }

    /// Lose access to the websocket but keep the callbacks in case any events are recieved.
    ///
    /// It's best not to use this function in production, as the callbacks and possibly the
    /// websocket itself will leak. TODO should this method exist at all?
    pub fn forget(mut self) { .. }

    /// Start the closing handshake.
    pub fn close(&self) { .. }

    /// Start the closing handshake, with a reason code and optional reason string.
    ///
    /// The code must be *1000* or between *3000* and *4999* inclusive, and the reason string, if
    /// present, must be less than or equal to 123 bytes in length (when utf8 encoded). If either
    /// of these conditions are violated, the function will error without closing the connection.
    pub fn close_with_reason(&self, code: u16, reason: Option<&str>) -> Result<(), CloseError> { .. }
}

impl std::ops::Drop for WebSocket {
    fn drop(&mut self) {
        // start the closing handshake
        ..
    }
}

/// An incoming websocket message.
pub enum Message {
    /// Message was in the binary variation.
    Binary(Vec<u8>),
    /// Message was in the text variation.
    Text(String),
}

impl Message {
    /// Utility method.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Message::Binary(msg) => Some(msg),
            Message::Text(_) => None,
        }
    }

    /// Utility method.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(msg) => Some(msg),
            Message::Binary(_) => None,
        }
    }
}

/// Information about the closing of the socket.
pub struct CloseEvent {
    /// Whether the websocket was shut down cleanly.
    pub was_clean: bool,
    /// The code representing the reason for the closure (see [the spec] for details).
    ///
    /// [the spec]: https://tools.ietf.org/html/rfc6455#page-45
    pub code: u16,
    /// A text description of the reason for the closure.
    pub reason: String,
}

/// The state of the websocket.
///
/// The websocket will usually transition between the states in order, but this is not always the
/// case.
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

impl State {
    /// Is this `State` `Connecting`.
    pub fn is_connecting(&self) -> bool { .. }

    /// Is this `State` `Open`.
    pub fn is_open(&self) -> bool { .. }

    /// Is this `State` `Closing`.
    pub fn is_closing(&self) -> bool { .. }

    /// Is this `State` `Closed`.
    pub fn is_closed(&self) -> bool { .. }
}

/// This error occurs only if the URL passed to `connect`, or the URL or protocols passed to
/// `connect_with_protocols` is malformed.
#[derive(Debug)]
pub struct ConnectError {
    msg: String,
}

impl fmt::Display for ConnectError { .. }
impl std::error::Error for ConnectError {}

/// There was an error closing the connection.
#[derive(Debug, Clone)]
pub enum CloseError {
    /// An invalid reason code was passed.
    InvalidCode(u16),
    /// An invalid reason string was passed.
    InvalidReason(String),
}

impl fmt::Display for CloseError { .. }
impl std::error::Error for CloseError {}

/// Attempted to send a message before the connection was established.
#[derive(Debug, Clone)]
pub struct SendError;

impl fmt::Display for SendError { .. }
impl std::error::Error for SendError {}

```

### The `futures` interface

```rust

pub struct WebSocket {
    inner: callback::WebSocket,
    state: Rc<RefCell<State>>,
}

impl Stream for WebSocket {
    type Item = Result<Message, ConnectionError>;

    ..
}

impl Sink<&str> for WebSocket {
    type Error = ConnectionError;

    ..
}

impl Sink<&[u8]> for WebSocket {
    type Error = ConnectionError;

    ..
}

impl WebSocket {
    pub fn connect(url: &str) -> Result<Self, ConnectError> { .. }
}

/// There was an error with the connection (the "error" event was fired).
#[derive(Debug)]
pub struct ConnectionError;
```

## Drawbacks, Rationale, and Alternatives

The main alternative here is that we could use a bespoke model for the futures websocket, rather than
using standard traits (Stream + Sink). I'm not sure exactly which would be preferable.

## Unresolved Questions

[HTML standard]: https://html.spec.whatwg.org/multipage/web-sockets.html
[protocol specification]: https://tools.ietf.org/html/rfc6455
[failure-information]: https://html.spec.whatwg.org/multipage/web-sockets.html#closeWebSocket

