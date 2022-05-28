use crate::worker::*;
use crate::{
    locate_callback_and_respond, Bridge, Callback, Discoverer, Dispatchable, HandlerId, Last,
    Shared, SharedOutputSlab, Worker, WorkerLifecycleEvent, WorkerLink, WorkerScope,
};
use anymap2::{self, AnyMap};
use queue::Queue;
use slab::Slab;
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

thread_local! {
    static REMOTE_WORKERS_POOL: RefCell<AnyMap> = RefCell::new(AnyMap::new());
    static QUEUE: Queue<TypeId> = Queue::new();
}

/// Create a single instance in a tab.
#[allow(missing_debug_implementations)]
pub struct Public<W> {
    _worker: PhantomData<W>,
}

impl<W, F> Discoverer for Public<W>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    type Worker = W;

    fn spawn_or_join(callback: Option<Callback<W::Output>>) -> Box<dyn Bridge<W>> {
        let bridge = REMOTE_WORKERS_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            match pool.entry::<RemoteWorker<W>>() {
                anymap2::Entry::Occupied(mut entry) => entry.get_mut().create_bridge(callback),
                anymap2::Entry::Vacant(entry) => {
                    let slab: Shared<Slab<Option<Callback<W::Output>>>> =
                        Rc::new(RefCell::new(Slab::new()));
                    let handler = {
                        let slab = slab.clone();
                        move |data: Vec<u8>, worker: &web_sys::Worker| {
                            let msg = FromWorker::<W::Output>::unpack(&data);
                            match msg {
                                FromWorker::WorkerLoaded => {
                                    QUEUE.with(|queue| {
                                        queue.insert_loaded_worker(TypeId::of::<W>());

                                        if let Some(msgs) =
                                            queue.remove_msg_queue(&TypeId::of::<W>())
                                        {
                                            for msg in msgs {
                                                worker.post_message_vec(msg)
                                            }
                                        }
                                    });
                                }
                                FromWorker::ProcessOutput(id, output) => {
                                    locate_callback_and_respond::<W>(&slab, id, output);
                                }
                            }
                        }
                    };
                    let name_of_resource = W::name_of_resource();
                    let is_relative = W::resource_path_is_relative();
                    let worker = {
                        let worker = worker_new(name_of_resource, is_relative, W::is_module());
                        let worker_clone = worker.clone();
                        worker.set_onmessage_closure(move |data: Vec<u8>| {
                            handler(data, &worker_clone);
                        });
                        worker
                    };
                    let launched = RemoteWorker::new(worker, slab);
                    entry.insert(launched).create_bridge(callback)
                }
            }
        });
        Box::new(bridge)
    }
}

impl<W, F> Dispatchable for Public<W>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
}

/// A connection manager for components interaction with workers.
pub struct PublicBridge<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    worker: web_sys::Worker,
    id: HandlerId,
    _worker: PhantomData<W>,
}

impl<W, F> fmt::Debug for PublicBridge<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PublicBridge<_>")
    }
}

impl<W, F> PublicBridge<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    /// Send a message to the worker, queuing the message if necessary
    fn send_message(&self, msg: ToWorker<W::Input>) {
        QUEUE.with(|queue| {
            if queue.is_worker_loaded(&TypeId::of::<W>()) {
                send_to_remote::<W>(&self.worker, msg);
            } else {
                queue.add_msg_to_queue(msg.pack(), TypeId::of::<W>());
            }
        });
    }
}

impl<W, F> Bridge<W> for PublicBridge<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    fn send(&mut self, msg: W::Input) {
        let msg = ToWorker::ProcessInput(self.id, msg);
        self.send_message(msg);
    }
}

impl<W, F> Drop for PublicBridge<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    fn drop(&mut self) {
        let terminate_worker = REMOTE_WORKERS_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            let terminate_worker = {
                if let Some(launched) = pool.get_mut::<RemoteWorker<W>>() {
                    launched.remove_bridge(self)
                } else {
                    false
                }
            };

            if terminate_worker {
                pool.remove::<RemoteWorker<W>>();
            }

            terminate_worker
        });

        let disconnected = ToWorker::Disconnected(self.id);
        self.send_message(disconnected);

        if terminate_worker {
            let destroy = ToWorker::Destroy;
            self.send_message(destroy);

            QUEUE.with(|queue| {
                queue.remove_worker(&TypeId::of::<W>());
            });
        }
    }
}

/// A trait to enable public workers being registered in a web worker.
pub trait PublicWorker {
    /// Executes an worker in the current environment.
    /// Uses in `main` function of a worker.
    fn register();
}

impl<W, F> PublicWorker for W
where
    W: Worker<Reach = Public<W>>,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
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

struct RemoteWorker<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    worker: web_sys::Worker,
    slab: SharedOutputSlab<W>,
}

impl<W, F> RemoteWorker<W, F>
where
    W: Worker,
    <W as Worker>::Input: SerDe<F>,
    <W as Worker>::Output: SerDe<F>,
{
    pub fn new(worker: web_sys::Worker, slab: SharedOutputSlab<W>) -> Self {
        RemoteWorker { worker, slab }
    }

    fn create_bridge(&mut self, callback: Option<Callback<W::Output>>) -> PublicBridge<W, F> {
        let respondable = callback.is_some();
        let mut slab = self.slab.borrow_mut();
        let id: usize = slab.insert(callback);
        let id = HandlerId::new(id, respondable);
        let bridge = PublicBridge {
            worker: self.worker.clone(),
            id,
            _worker: PhantomData,
        };
        bridge.send_message(ToWorker::Connected(bridge.id));

        bridge
    }

    fn remove_bridge(&mut self, bridge: &PublicBridge<W, F>) -> Last {
        let mut slab = self.slab.borrow_mut();
        let _ = slab.remove(bridge.id.raw_id());
        slab.is_empty()
    }
}
