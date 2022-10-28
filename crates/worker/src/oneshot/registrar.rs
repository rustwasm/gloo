use std::fmt;

use super::traits::Oneshot;
use super::worker::OneshotWorker;
use crate::actor::WorkerRegistrar;
use crate::codec::{Bincode, Codec};
use crate::traits::Registrable;

/// A registrar for oneshot workers.
pub struct OneshotRegistrar<T, CODEC = Bincode>
where
    T: Oneshot + 'static,
    CODEC: Codec + 'static,
{
    inner: WorkerRegistrar<OneshotWorker<T>, CODEC>,
}

impl<T, CODEC> Default for OneshotRegistrar<T, CODEC>
where
    T: Oneshot + 'static,
    CODEC: Codec + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, CODEC> OneshotRegistrar<T, CODEC>
where
    T: Oneshot + 'static,
    CODEC: Codec + 'static,
{
    /// Creates a new Oneshot Registrar.
    pub fn new() -> Self {
        Self {
            inner: OneshotWorker::<T>::registrar().encoding::<CODEC>(),
        }
    }

    /// Sets the encoding.
    pub fn encoding<C>(&self) -> OneshotRegistrar<T, C>
    where
        C: Codec + 'static,
    {
        OneshotRegistrar {
            inner: self.inner.encoding::<C>(),
        }
    }

    /// Registers the agent.
    pub fn register(&self) {
        self.inner.register()
    }
}

impl<T, CODEC> fmt::Debug for OneshotRegistrar<T, CODEC>
where
    T: Oneshot + 'static,
    CODEC: Codec + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OneshotRegistrar<_>").finish()
    }
}
