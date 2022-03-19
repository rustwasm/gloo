use serde::{Deserialize, Serialize};

use crate::handler_id::HandlerId;
use crate::scope::WorkerScope;
use crate::spawner::WorkerSpawner;

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

    /// Receives an update.
    fn update(&mut self, msg: Self::Message);

    /// This method called on when a new bridge created.
    fn connected(&mut self, _id: HandlerId) {}

    /// Receives an input.
    fn received(&mut self, msg: Self::Input, id: HandlerId);

    /// This method called on when a new bridge destroyed.
    fn disconnected(&mut self, _id: HandlerId) {}

    /// This method called when the worker is destroyed.
    fn destroy(&mut self) {}
}

/// A Worker that can be spawned by a spawner.
pub trait Spawnable: Worker {
    /// Creates a spawner.
    fn spawner() -> WorkerSpawner<Self>;
}

impl<T> Spawnable for T
where
    T: Worker,
{
    fn spawner() -> WorkerSpawner<Self> {
        WorkerSpawner::new()
    }
}
