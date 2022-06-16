use serde::{Deserialize, Serialize};

use crate::handler_id::HandlerId;
use crate::registrar::WorkerRegistrar;
use crate::scope::{WorkerDestroyHandle, WorkerScope};
use crate::spawner::WorkerSpawner;

/// Declares the behaviour of a worker.
pub trait Worker: Sized + 'static {
    /// Update message type.
    type Message;
    /// Incoming message type.
    type Input: Serialize + for<'de> Deserialize<'de>;
    /// Outgoing message type.
    type Output: Serialize + for<'de> Deserialize<'de>;

    /// Creates an instance of a worker.
    fn create(scope: &WorkerScope<Self>) -> Self;

    /// Receives an update.
    ///
    /// This method is called when the worker send messages to itself via [`WorkerScope::send_message`].
    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message);

    /// New bridge created.
    ///
    /// When a new bridge is created by [`WorkerSpawner::spawn`](crate::spawner::WorkerSpawner)
    /// or [`WorkerBridge::fork`](crate::WorkerBridge::fork),
    /// the worker will be notified the [`HandlerId`] of the created bridge via this method.
    fn connected(&mut self, scope: &WorkerScope<Self>, id: HandlerId) {
        let _scope = scope;
        let _id = id;
    }

    /// Receives an input from a connected bridge.
    ///
    /// When a bridge sends an input via [`WorkerBridge::send`](crate::WorkerBridge::send), the worker will receive the
    /// input via this method.
    fn received(&mut self, scope: &WorkerScope<Self>, msg: Self::Input, id: HandlerId);

    /// Existing bridge destroyed.
    ///
    /// When a bridge is dropped, the worker will be notified with this method.
    fn disconnected(&mut self, scope: &WorkerScope<Self>, id: HandlerId) {
        let _scope = scope;
        let _id = id;
    }

    /// Destroys the current worker.
    ///
    /// When all bridges are dropped, the method will be invoked.
    ///
    /// This method is provided a destroy handle where when it is dropped, the worker is closed.
    /// If the worker is closed immediately, then it can ignore the destroy handle.
    /// Otherwise hold the destroy handle until the clean up task is finished.
    ///
    /// # Note
    ///
    /// This method will only be called after all bridges are disconnected.
    /// Attempting to send messages after this method is called will have no effect.
    fn destroy(&mut self, scope: &WorkerScope<Self>, destruct: WorkerDestroyHandle<Self>) {
        let _scope = scope;
        let _destruct = destruct;
    }
}

/// A Worker that can be spawned by a spawner.
pub trait Spawnable {
    /// Spawner Type.
    type Spawner;

    /// Creates a spawner.
    fn spawner() -> Self::Spawner;
}

impl<T> Spawnable for T
where
    T: Worker,
{
    type Spawner = WorkerSpawner<Self>;

    fn spawner() -> WorkerSpawner<Self> {
        WorkerSpawner::new()
    }
}

/// A trait to enable public workers being registered in a web worker.
pub trait Registrable {
    /// Registrar Type.
    type Registrar;

    /// Creates a registrar for the current worker.
    fn registrar() -> Self::Registrar;
}

impl<W> Registrable for W
where
    W: Worker,
{
    type Registrar = WorkerRegistrar<Self>;

    fn registrar() -> WorkerRegistrar<Self> {
        WorkerRegistrar::new()
    }
}
