use std::cell::RefCell;
use std::fmt;
#[cfg(feature = "futures")]
use std::future::Future;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use crate::handler_id::HandlerId;
use crate::messages::{FromWorker, Packed};
use crate::traits::Worker;
use crate::worker_ext::{worker_self, WorkerExt};
use crate::Shared;

/// This struct holds a reference to a component and to a global scheduler.
pub struct WorkerScope<W: Worker> {
    state: Shared<WorkerState<W>>,
}

impl<W: Worker> fmt::Debug for WorkerScope<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerScope<_>")
    }
}

impl<W: Worker> Clone for WorkerScope<W> {
    fn clone(&self) -> Self {
        WorkerScope {
            state: self.state.clone(),
        }
    }
}

impl<W> WorkerScope<W>
where
    W: Worker,
{
    /// Create worker scope
    pub(crate) fn new() -> Self {
        let state = Rc::new(RefCell::new(WorkerState::new()));
        WorkerScope { state }
    }

    /// Schedule message for sending to worker
    pub(crate) fn send(&self, event: WorkerLifecycleEvent<W>) {
        WorkerRunnable {
            state: self.state.clone(),
            event,
        }
        .run();
    }

    /// Send response to a worker bridge.
    pub fn respond(&self, id: HandlerId, output: W::Output) {
        let msg = FromWorker::<W>::ProcessOutput(id, output);
        let data = msg.pack();
        worker_self().post_message_vec(data);
    }

    /// Send a message to the worker
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<W::Message>,
    {
        self.send(WorkerLifecycleEvent::Message(msg.into()));
    }

    /// Create a callback which will send a message to the worker when invoked.
    pub fn callback<F, IN, M>(&self, function: F) -> Rc<dyn Fn(IN)>
    where
        M: Into<W::Message>,
        F: Fn(IN) -> M + 'static,
    {
        let scope = self.clone();
        let closure = move |input| {
            let output = function(input).into();
            scope.send(WorkerLifecycleEvent::Message(output));
        };
        Rc::new(closure)
    }

    /// This method creates a callback which returns a Future which
    /// returns a message to be sent back to the worker
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and
    /// will leak.
    #[cfg(feature = "futures")]
    #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
    pub fn callback_future<FN, FU, IN, M>(&self, function: FN) -> Rc<dyn Fn(IN)>
    where
        M: Into<W::Message>,
        FU: Future<Output = M> + 'static,
        FN: Fn(IN) -> FU + 'static,
    {
        let link = self.clone();

        let closure = move |input: IN| {
            let future: FU = function(input);
            link.send_future(future);
        };

        Rc::new(closure)
    }

    /// This method processes a Future that returns a message and sends it back to the worker.
    ///
    /// # Panics
    /// If the future panics, then the promise will not resolve, and will leak.
    #[cfg(feature = "futures")]
    #[cfg_attr(docsrs, doc(cfg(feature = "futures")))]
    pub fn send_future<F, M>(&self, future: F)
    where
        M: Into<W::Message>,
        F: Future<Output = M> + 'static,
    {
        let link = self.clone();
        let js_future = async move {
            let message: W::Message = future.await.into();
            let cb = link.callback(|m: W::Message| m);
            (*cb)(message);
        };
        wasm_bindgen_futures::spawn_local(js_future);
    }
}

impl<W: Worker> Default for WorkerScope<W> {
    fn default() -> Self {
        Self::new()
    }
}

struct WorkerState<W> {
    worker: Option<W>,
    // TODO: Use worker field to control create message this flag
    destroyed: bool,
}

impl<W> WorkerState<W> {
    fn new() -> Self {
        WorkerState {
            worker: None,
            destroyed: false,
        }
    }
}

/// Internal Worker lifecycle events
#[derive(Debug)]
pub(crate) enum WorkerLifecycleEvent<W: Worker> {
    /// Request to create link
    Create(WorkerScope<W>),
    /// Internal Worker message
    Message(W::Message),
    /// Client connected
    Connected(HandlerId),
    /// Received message from Client
    Input(W::Input, HandlerId),
    /// Client disconnected
    Disconnected(HandlerId),
    /// Request to destroy worker
    Destroy,
}

struct WorkerRunnable<W: Worker> {
    state: Shared<WorkerState<W>>,
    event: WorkerLifecycleEvent<W>,
}

impl<W> WorkerRunnable<W>
where
    W: Worker,
{
    fn run(self) {
        let mut state = self.state.borrow_mut();
        if state.destroyed {
            return;
        }
        match self.event {
            WorkerLifecycleEvent::Create(link) => {
                state.worker = Some(W::create(link));
            }
            WorkerLifecycleEvent::Message(msg) => {
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to process messages")
                    .update(msg);
            }
            WorkerLifecycleEvent::Connected(id) => {
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to send a connected message")
                    .connected(id);
            }
            WorkerLifecycleEvent::Input(inp, id) => {
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to process inputs")
                    .handle_input(inp, id);
            }
            WorkerLifecycleEvent::Disconnected(id) => {
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to send a disconnected message")
                    .disconnected(id);
            }
            WorkerLifecycleEvent::Destroy => {
                let mut worker = state
                    .worker
                    .take()
                    .expect_throw("trying to destroy not existent worker");
                worker.destroy();
                state.destroyed = true;
            }
        }
    }
}
