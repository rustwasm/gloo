use futures::stream::Stream;
use serde::de::Deserialize;
use serde::ser::Serialize;

use super::worker::ReactorWorker;
use super::{Reactor, ReactorBridge};
use crate::actor::WorkerSpawner;
use crate::codec::{Bincode, Codec};

/// A spawner to create oneshot workers.
#[derive(Debug, Default)]
pub struct ReactorSpawner<R, CODEC = Bincode>
where
    R: Reactor + 'static,
    CODEC: Codec,
{
    inner: WorkerSpawner<ReactorWorker<R>, CODEC>,
}

impl<R, CODEC> ReactorSpawner<R, CODEC>
where
    R: Reactor + 'static,
    CODEC: Codec,
{
    /// Creates a ReactorSpawner.
    pub const fn new() -> Self {
        Self {
            inner: WorkerSpawner::<ReactorWorker<R>, CODEC>::new(),
        }
    }

    /// Sets a new message encoding.
    pub const fn encoding<C>(&self) -> ReactorSpawner<R, C>
    where
        C: Codec,
    {
        ReactorSpawner {
            inner: WorkerSpawner::<ReactorWorker<R>, C>::new(),
        }
    }

    /// Spawns a reactor worker.
    pub fn spawn(mut self, path: &str) -> ReactorBridge<R>
    where
        <R::InputStream as Stream>::Item: Serialize + for<'de> Deserialize<'de>,
        <R::OutputStream as Stream>::Item: Serialize + for<'de> Deserialize<'de>,
    {
        let rx = ReactorBridge::register_callback(&mut self.inner);

        let inner = self.inner.spawn(path);

        ReactorBridge::new(inner, rx)
    }
}
