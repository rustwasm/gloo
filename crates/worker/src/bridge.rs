use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;
use std::rc::Weak;

use crate::handler_id::HandlerId;
use crate::messages::ToWorker;
use crate::native_worker::{DedicatedWorker, NativeWorkerExt};
use crate::traits::Worker;
use crate::{Callback, Shared};

pub(crate) type ToWorkerQueue<W> = Vec<ToWorker<W>>;
pub(crate) type CallbackMap<W> = HashMap<HandlerId, Weak<dyn Fn(<W as Worker>::Output)>>;

struct BridgeInner<W>
where
    W: Worker,
{
    worker: DedicatedWorker,
    // When worker is loaded, queue becomes None.
    pending_queue: Shared<Option<ToWorkerQueue<W>>>,

    callbacks: Shared<CallbackMap<W>>,
}

impl<W> fmt::Debug for BridgeInner<W>
where
    W: Worker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BridgeInner<_>")
    }
}

impl<W> BridgeInner<W>
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
                self.worker.post_packed_message(msg);
            }
        }
    }
}

impl<W> Drop for BridgeInner<W>
where
    W: Worker,
{
    fn drop(&mut self) {
        let destroy = ToWorker::Destroy;
        self.send_message(destroy);
    }
}

/// A connection manager for components interaction with workers.
pub struct Bridge<W>
where
    W: Worker,
{
    inner: Rc<BridgeInner<W>>,
    id: HandlerId,
    _worker: PhantomData<W>,
    cb: Option<Rc<dyn Fn(W::Output)>>,
}

impl<W> Bridge<W>
where
    W: Worker,
{
    pub(crate) fn new(
        id: HandlerId,
        native_worker: web_sys::Worker,
        pending_queue: Rc<RefCell<Option<ToWorkerQueue<W>>>>,
        callbacks: Rc<RefCell<CallbackMap<W>>>,
        callback: Option<Callback<W::Output>>,
    ) -> Self {
        Self {
            inner: BridgeInner {
                worker: native_worker,
                pending_queue,
                callbacks,
            }
            .into(),
            id,
            _worker: PhantomData,
            cb: callback,
        }
    }

    /// Send a message to the current worker.
    pub fn send(&self, msg: W::Input) {
        let msg = ToWorker::ProcessInput(self.id, msg);
        self.inner.send_message(msg);
    }

    /// Forks the bridge with a different callback.
    ///
    /// This creates a new HandlerID that helps the worker to differentiate bridges.
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

        Self {
            inner: self.inner.clone(),
            id: handler_id,
            _worker: PhantomData,
            cb,
        }
    }
}

impl<W> Clone for Bridge<W>
where
    W: Worker,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            id: self.id,
            _worker: PhantomData,
            cb: self.cb.clone(),
        }
    }
}

impl<W> Drop for Bridge<W>
where
    W: Worker,
{
    fn drop(&mut self) {
        let disconnected = ToWorker::Disconnected(self.id);
        self.inner.send_message(disconnected);
    }
}

impl<W> fmt::Debug for Bridge<W>
where
    W: Worker,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Bridge<_>")
    }
}
