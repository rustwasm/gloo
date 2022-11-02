use crate::actor::WorkerSpawner;
use crate::codec::{Bincode, Codec};

use super::worker::OneshotWorker;
use super::{Oneshot, OneshotBridge};

/// A spawner to create oneshot workers.
#[derive(Debug, Default)]
pub struct OneshotSpawner<N, CODEC = Bincode>
where
    N: Oneshot + 'static,
    CODEC: Codec,
{
    inner: WorkerSpawner<OneshotWorker<N>, CODEC>,
}

impl<N, CODEC> OneshotSpawner<N, CODEC>
where
    N: Oneshot + 'static,
    CODEC: Codec,
{
    /// Creates a [OneshotSpawner].
    pub fn new() -> Self {
        Self {
            inner: WorkerSpawner::<OneshotWorker<N>, CODEC>::new(),
        }
    }

    /// Sets a new message encoding.
    pub fn encoding<C>(&mut self) -> OneshotSpawner<N, C>
    where
        C: Codec,
    {
        OneshotSpawner {
            inner: WorkerSpawner::<OneshotWorker<N>, C>::new(),
        }
    }

    /// Spawns an Oneshot Worker.
    pub fn spawn(mut self, path: &str) -> OneshotBridge<N> {
        let rx = OneshotBridge::register_callback(&mut self.inner);

        let inner = self.inner.spawn(path);

        OneshotBridge::new(inner, rx)
    }
}
