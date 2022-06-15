use std::marker::PhantomData;

use crate::codec::{Bincode, Codec};
use crate::lifecycle::WorkerLifecycleEvent;
use crate::messages::{FromWorker, ToWorker};
use crate::native_worker::{DedicatedWorker, NativeWorkerExt, WorkerSelf};
use crate::scope::WorkerScope;
use crate::traits::Worker;

/// A trait to enable public workers being registered in a web worker.
pub trait Registrable: Worker {
    /// Creates a registrar for the current worker.
    fn registrar() -> Registrar<Self>;
}

impl<W> Registrable for W
where
    W: Worker,
{
    fn registrar() -> Registrar<Self> {
        Registrar {
            _marker: PhantomData,
        }
    }
}

/// A Worker Registrar.
pub struct Registrar<W, CODEC = Bincode>
where
    W: Worker,
    CODEC: Codec,
{
    _marker: PhantomData<(W, CODEC)>,
}

impl<W, CODEC> Registrar<W, CODEC>
where
    W: Worker,
    CODEC: Codec,
{
    /// Sets a new message encoding
    pub fn encoding<C>(&self) -> Registrar<W, C>
    where
        C: Codec,
    {
        Registrar {
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
