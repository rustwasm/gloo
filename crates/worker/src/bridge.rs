use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::messages::ToWorker;
use crate::worker_ext::WorkerExt;
use crate::{HandlerId, Worker};

type ToWorkerQueue<W> = Vec<ToWorker<W>>;

struct WorkerBridgeInner<W>
where
    W: Worker,
{
    worker: web_sys::Worker,
    // When worker is loaded, queue becomes None.
    pending_queue: Rc<RefCell<Option<ToWorkerQueue<W>>>>,
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
                self.worker.post_to_worker::<W>(msg);
            }
        }
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
#[derive(Debug, Clone)]
pub struct WorkerBridge<W>
where
    W: Worker,
{
    inner: Rc<WorkerBridgeInner<W>>,
    id: HandlerId,
    _worker: PhantomData<W>,
}

impl<W> WorkerBridge<W>
where
    W: Worker,
{
    pub fn send(&mut self, msg: W::Input) {
        let msg = ToWorker::ProcessInput(self.id, msg);
        self.inner.send_message(msg);
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
