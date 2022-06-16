use std::marker::PhantomData;

use crate::codec::{Bincode, Codec};
use crate::lifecycle::WorkerLifecycleEvent;
use crate::messages::{FromWorker, ToWorker};
use crate::native_worker::{DedicatedWorker, NativeWorkerExt, WorkerSelf};
use crate::scope::WorkerScope;
use crate::traits::Worker;

/// A Worker Registrar.
pub struct WorkerRegistrar<W, CODEC = Bincode>
where
    W: Worker,
    CODEC: Codec,
{
    _marker: PhantomData<(W, CODEC)>,
}

impl<W, CODEC> WorkerRegistrar<W, CODEC>
where
    W: Worker,
    CODEC: Codec,
{
    pub(crate) fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Sets a new message encoding
    pub fn encoding<C>(&self) -> WorkerRegistrar<W, C>
    where
        C: Codec,
    {
        WorkerRegistrar {
            _marker: PhantomData,
        }
    }

    /// Executes an worker in the current environment.
    pub fn register(&self)
    where
        CODEC: Codec,
    {
        let scope = WorkerScope::<W>::new::<CODEC>();
        let upd = WorkerLifecycleEvent::Create(scope.clone());
        scope.send(upd);
        let handler = move |msg: ToWorker<W>| {
            let upd = WorkerLifecycleEvent::Remote(msg);
            scope.send(upd);
        };
        let loaded: FromWorker<W> = FromWorker::WorkerLoaded;
        let worker = DedicatedWorker::worker_self();
        worker.set_on_packed_message::<_, CODEC, _>(handler);
        worker.post_packed_message::<_, CODEC>(loaded);
    }
}
