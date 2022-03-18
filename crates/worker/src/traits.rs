use serde::{Deserialize, Serialize};

use crate::handler_id::HandlerId;
use crate::scope::WorkerScope;

/// Declares the behavior of the worker.
pub trait Worker: Sized + 'static {
    /// Type of an input message.
    type Message;
    /// Incoming message type.
    type Input: Serialize + for<'de> Deserialize<'de>;
    /// Outgoing message type.
    type Output: Serialize + for<'de> Deserialize<'de>;

    /// Creates an instance of an worker.
    fn create(link: WorkerScope<Self>) -> Self;

    /// Receives an update via [Message].
    fn update(&mut self, msg: Self::Message);

    /// This method called on when a new bridge created.
    fn connected(&mut self, _id: HandlerId) {}

    /// This method called on every incoming message.
    fn handle_input(&mut self, msg: Self::Input, id: HandlerId);

    /// This method called on when a new bridge destroyed.
    fn disconnected(&mut self, _id: HandlerId) {}

    /// This method called when the worker is destroyed.
    fn destroy(&mut self) {}
}
