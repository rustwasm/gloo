//! WebSocket Events

/// This is created from [`ErrorEvent`][web_sys::ErrorEvent] received from `onerror` listener of the WebSocket.
#[derive(Clone, Debug)]
pub struct ErrorEvent {
    /// The error message.
    pub message: String,
}

/// Data emiited by `onclose` event
#[derive(Clone, Debug)]
pub struct CloseEvent {
    /// Close code
    pub code: u16,
    /// Close reason
    pub reason: String,
    /// If the websockt was closed cleanly
    pub was_clean: bool,
}
