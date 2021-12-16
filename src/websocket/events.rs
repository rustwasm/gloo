//! WebSocket Events

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
