use crate::worker::*;
use crate::{
    Bridge, Callback, Discoverer, HandlerId, Worker, WorkerLifecycleEvent, WorkerLink, WorkerScope,
};
use queue::Queue;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

thread_local! {
    static QUEUE: Queue<usize> = Queue::new();
}

static PRIVATE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
const SINGLETON_ID: HandlerId = HandlerId(0, true);

/// Create a new instance for every bridge.
#[allow(missing_debug_implementations)]
pub struct Private<W> {
    _worker: PhantomData<W>,
}

/// A trait to enable private workers being registered in a web worker.
pub trait PrivateWorker {
    /// Executes an worker in the current environment.
    /// Uses in `main` function of a worker.
    fn register();
}

impl<W> PrivateWorker for W
where
    W: Worker<Reach = Private<W>>,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn register() {
        let scope = WorkerScope::<W>::new();
        let responder = WorkerResponder;
        let link = WorkerLink::connect(&scope, responder);
        let upd = WorkerLifecycleEvent::Create(link);
        scope.send(upd);
        let handler = move |data: Vec<u8>| {
            let msg = ToWorker::<W::Input>::unpack(&data);
            match msg {
                ToWorker::Connected(_) => {
                    let upd = WorkerLifecycleEvent::Connected(SINGLETON_ID);
                    scope.send(upd);
                }
                ToWorker::ProcessInput(_, value) => {
                    let upd = WorkerLifecycleEvent::Input(value, SINGLETON_ID);
                    scope.send(upd);
                }
                ToWorker::Disconnected(_) => {
                    let upd = WorkerLifecycleEvent::Disconnected(SINGLETON_ID);
                    scope.send(upd);
                }
                ToWorker::Destroy => {
                    let upd = WorkerLifecycleEvent::Destroy;
                    scope.send(upd);
                    // Terminates web worker
                    worker_self().close();
                }
            }
        };
        let loaded: FromWorker<W::Output> = FromWorker::WorkerLoaded;
        let loaded = loaded.pack();
        let worker = worker_self();
        worker.set_onmessage_closure(handler);
        worker.post_message_vec(loaded);
    }
}

impl<W> Discoverer for Private<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    type Worker = W;

    fn spawn_or_join(callback: Option<Callback<W::Output>>) -> Box<dyn Bridge<W>> {
        let id = PRIVATE_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let callback = callback.expect("Callback required for Private workers");
        let handler = move |data: Vec<u8>, worker: &web_sys::Worker| {
            let msg = FromWorker::<W::Output>::unpack(&data);
            match msg {
                FromWorker::WorkerLoaded => {
                    QUEUE.with(|queue| {
                        queue.insert_loaded_worker(id);

                        if let Some(msgs) = queue.remove_msg_queue(&id) {
                            for msg in msgs {
                                worker.post_message_vec(msg)
                            }
                        }
                    });
                }
                FromWorker::ProcessOutput(id, output) => {
                    assert_eq!(id.raw_id(), SINGLETON_ID.raw_id());
                    (*callback)(output);
                }
            }
        };

        let name_of_resource = W::name_of_resource();
        let is_relative = W::resource_path_is_relative();

        let worker = {
            let worker = worker_new(name_of_resource, is_relative, W::is_module());
            let worker_clone = worker.clone();
            worker.set_onmessage_closure(move |data: Vec<u8>| handler(data, &worker_clone));
            worker
        };
        let bridge = PrivateBridge {
            worker,
            _worker: PhantomData,
            id,
        };
        bridge.send_message(ToWorker::Connected(SINGLETON_ID));
        Box::new(bridge)
    }
}

/// A connection manager for components interaction with workers.
pub struct PrivateBridge<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    worker: web_sys::Worker,
    _worker: PhantomData<W>,
    id: usize,
}

impl<W> PrivateBridge<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    /// Send a message to the worker, queuing the message if necessary
    fn send_message(&self, msg: ToWorker<W::Input>) {
        QUEUE.with(|queue| {
            if queue.is_worker_loaded(&self.id) {
                send_to_remote::<W>(&self.worker, msg);
            } else {
                queue.add_msg_to_queue(msg.pack(), self.id);
            }
        });
    }
}

impl<W> fmt::Debug for PrivateBridge<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PrivateBridge<_>")
    }
}

impl<W> Bridge<W> for PrivateBridge<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn send(&mut self, msg: W::Input) {
        let msg = ToWorker::ProcessInput(SINGLETON_ID, msg);
        self.send_message(msg);
    }
}

impl<W> Drop for PrivateBridge<W>
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn drop(&mut self) {
        self.worker.set_onmessage_closure(|_| {});

        let disconnected = ToWorker::Disconnected(SINGLETON_ID);
        send_to_remote::<W>(&self.worker, disconnected);

        let destroy = ToWorker::Destroy;
        send_to_remote::<W>(&self.worker, destroy);

        QUEUE.with(|queue| {
            queue.remove_worker(&self.id);
        });
    }
}
