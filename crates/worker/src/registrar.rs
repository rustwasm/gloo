use crate::messages::{FromWorker, Packed, ToWorker};
use crate::native_worker::{DedicatedWorker, NativeWorkerExt, WorkerSelf};
use crate::scope::{WorkerLifecycleEvent, WorkerScope};
use crate::traits::Worker;

/// A trait to enable public workers being registered in a web worker.
pub trait WorkerRegistrar {
    /// Executes an worker in the current environment.
    /// Uses in `main` function of a worker.
    fn register();
}

impl<W> WorkerRegistrar for W
where
    W: Worker,
{
    fn register() {
        let scope = WorkerScope::<W>::new();
        let upd = WorkerLifecycleEvent::Create(scope.clone());
        scope.send(upd);
        let handler = move |data: Vec<u8>| {
            let msg = ToWorker::<W>::unpack(&data);
            match msg {
                ToWorker::Connected(id) => {
                    let upd = WorkerLifecycleEvent::Connected(id);
                    scope.send(upd);
                }
                ToWorker::ProcessInput(id, value) => {
                    let upd = WorkerLifecycleEvent::Input(value, id);
                    scope.send(upd);
                }
                ToWorker::Disconnected(id) => {
                    let upd = WorkerLifecycleEvent::Disconnected(id);
                    scope.send(upd);
                }
                ToWorker::Destroy => {
                    let upd = WorkerLifecycleEvent::Destroy;
                    scope.send(upd);
                    // Terminates web worker
                    DedicatedWorker::worker_self().close();
                }
            }
        };
        let loaded: FromWorker<W> = FromWorker::WorkerLoaded;
        let worker = DedicatedWorker::worker_self();
        worker.set_on_packed_message(handler);
        worker.post_packed_message(loaded);
    }
}
