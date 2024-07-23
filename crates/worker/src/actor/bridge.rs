use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::rc::Weak;

use serde::{Deserialize, Serialize};

use super::handler_id::HandlerId;
use super::messages::FromWorker;
use super::messages::ToWorker;
use super::native_worker::DedicatedWorker;
use super::native_worker::NativeWorkerExt;
use super::traits::Worker;
use super::Callback;
use crate::codec::Codec;

pub(crate) type ToWorkerQueue<W> = Vec<ToWorker<W>>;
pub(crate) type CallbackMap<W> = HashMap<HandlerId, Weak<dyn Fn(<W as Worker>::Output)>>;
type PostMsg<W> = Box<dyn Fn(&DedicatedWorker, ToWorker<W>)>;

pub(crate) struct WorkerBridgeInner<W>
where
    W: Worker,
{
    // When worker is loaded, queue becomes None.
    pending_queue: RefCell<Option<ToWorkerQueue<W>>>,
    callbacks: RefCell<CallbackMap<W>>,
    native_worker: RefCell<Option<DedicatedWorker>>,
    post_msg: PostMsg<W>,
}

impl<W> WorkerBridgeInner<W>
where
    W: Worker + 'static,
{
    pub(crate) fn new<CODEC>(native_worker: DedicatedWorker, callbacks: CallbackMap<W>) -> Rc<Self>
    where
        CODEC: Codec,
        W::Input: Serialize + for<'de> Deserialize<'de>,
        W::Output: Serialize + for<'de> Deserialize<'de>,
    {
        let worker = native_worker.clone();

        let pending_queue = RefCell::new(Some(Vec::new()));
        let callbacks = RefCell::new(callbacks);
        let native_worker = RefCell::new(Some(native_worker));
        let post_msg = move |worker: &DedicatedWorker, msg: ToWorker<W>| {
            worker.post_packed_message::<_, CODEC>(msg)
        };

        let self_ = Self {
            pending_queue,
            callbacks,
            native_worker,
            post_msg: Box::new(post_msg),
        };
        let self_ = Rc::new(self_);

        let handler = {
            let bridge_inner = Rc::downgrade(&self_);
            // If all bridges are dropped then `self_` is dropped and `upgrade` returns `None`.
            move |msg: FromWorker<W>| {
                if let Some(bridge_inner) = Weak::upgrade(&bridge_inner) {
                    match msg {
                        FromWorker::WorkerLoaded => {
                            // Set pending queue to `None`. Unless `WorkerLoaded` is
                            // sent twice, this will always be `Some`.
                            if let Some(pending_queue) = bridge_inner.take_queue() {
                                // Will be `None` if the worker has been terminated.
                                if let Some(worker) =
                                    bridge_inner.native_worker.borrow_mut().as_ref()
                                {
                                    // Send all pending messages.
                                    for to_worker in pending_queue.into_iter() {
                                        (bridge_inner.post_msg)(worker, to_worker);
                                    }
                                }
                            }
                        }
                        FromWorker::ProcessOutput(id, output) => {
                            let mut callbacks = bridge_inner.callbacks.borrow_mut();

                            if let Some(m) = callbacks.get(&id) {
                                if let Some(m) = Weak::upgrade(m) {
                                    m(output);
                                } else {
                                    // The bridge has been dropped.
                                    callbacks.remove(&id);
                                }
                            }
                        }
                    }
                }
            }
        };

        worker.set_on_packed_message::<_, CODEC, _>(handler);

        self_
    }

    fn take_queue(&self) -> Option<ToWorkerQueue<W>> {
        self.pending_queue.borrow_mut().take()
    }
}

impl<W> fmt::Debug for WorkerBridgeInner<W>
where
    W: Worker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerBridgeInner<_>")
    }
}

impl<W> WorkerBridgeInner<W>
where
    W: Worker,
{
    /// Send a message to the worker, queuing the message if necessary
    fn send_message(&self, msg: ToWorker<W>) {
        let mut pending_queue = self.pending_queue.borrow_mut();

        match pending_queue.as_mut() {
            Some(m) => {
                m.push(msg);
            }
            None => {
                if let Some(worker) = self.native_worker.borrow().as_ref() {
                    (self.post_msg)(worker, msg);
                }
            }
        }
    }

    /// Terminate the worker, no more messages can be sent after this.
    fn terminate(&self) {
        if let Some(worker) = self.native_worker.borrow_mut().take() {
            worker.terminate();
        }
    }

    /// Returns true if the worker is terminated.
    fn is_terminated(&self) -> bool {
        self.native_worker.borrow().is_none()
    }
}

impl<W> Drop for WorkerBridgeInner<W>
where
    W: Worker,
{
    fn drop(&mut self) {
        let destroy = ToWorker::Destroy;
        self.send_message(destroy);
    }
}

/// A connection manager for components interaction with workers.
///
/// Dropping this object will send a disconnect message to the worker and drop
/// the callback if set, but will have no effect on forked bridges. Note that
/// the worker will still receive and process any messages sent over the bridge
/// up to that point, but the reply will not trigger a callback. If all forked
/// bridges for a worker are dropped, the worker will be sent a destroy message.
///
/// To terminate the worker and stop execution immediately, use
/// [`terminate`](#method.terminate).
pub struct WorkerBridge<W>
where
    W: Worker,
{
    inner: Rc<WorkerBridgeInner<W>>,
    id: HandlerId,
    _worker: PhantomData<W>,
    _cb: Option<Rc<dyn Fn(W::Output)>>,
}

impl<W> WorkerBridge<W>
where
    W: Worker,
{
    fn init(&self) {
        self.inner.send_message(ToWorker::Connected(self.id));
    }

    pub(crate) fn new(
        id: HandlerId,
        inner: Rc<WorkerBridgeInner<W>>,
        callback: Option<Callback<W::Output>>,
    ) -> Self
    where
        W::Input: Serialize + for<'de> Deserialize<'de>,
    {
        let self_ = Self {
            inner,
            id,
            _worker: PhantomData,
            _cb: callback,
        };
        self_.init();

        self_
    }

    /// Send a message to the current worker.
    pub fn send(&self, msg: W::Input) {
        let msg = ToWorker::ProcessInput(self.id, msg);
        self.inner.send_message(msg);
    }

    /// Forks the bridge with a different callback.
    ///
    /// This creates a new [HandlerID] that helps the worker to differentiate bridges.
    pub fn fork<F>(&self, cb: Option<F>) -> Self
    where
        F: 'static + Fn(W::Output),
    {
        let cb = cb.map(|m| Rc::new(m) as Rc<dyn Fn(W::Output)>);
        let handler_id = HandlerId::new();

        if let Some(cb_weak) = cb.as_ref().map(Rc::downgrade) {
            self.inner
                .callbacks
                .borrow_mut()
                .insert(handler_id, cb_weak);
        }

        let self_ = Self {
            inner: self.inner.clone(),
            id: handler_id,
            _worker: PhantomData,
            _cb: cb,
        };
        self_.init();

        self_
    }

    /// Immediately terminates the worker and stops any execution in progress,
    /// for this and all forked bridges. All messages will be dropped without
    /// the worker receiving them. No disconnect or destroy message is sent. Any
    /// messages sent after this point are dropped (from this bridge or any
    /// forks).
    ///
    /// For more details see
    /// [`web_sys::Worker::terminate`](https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Worker.html#method.terminate).
    pub fn terminate(&self) {
        self.inner.terminate()
    }

    /// Returns true if the worker is terminated.
    pub fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

impl<W> Drop for WorkerBridge<W>
where
    W: Worker,
{
    fn drop(&mut self) {
        let disconnected = ToWorker::Disconnected(self.id);
        self.inner.send_message(disconnected);
    }
}

impl<W> fmt::Debug for WorkerBridge<W>
where
    W: Worker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerBridge<_>")
    }
}

impl<W> PartialEq for WorkerBridge<W>
where
    W: Worker,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.id == rhs.id
    }
}
