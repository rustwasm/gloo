use crate::Shared;
use crate::{HandlerId, Worker};
use std::cell::RefCell;
use std::fmt;
#[cfg(feature = "futures")]
use std::future::Future;
use std::rc::Rc;

/// Defines communication from Worker to Consumers
pub(crate) trait Responder<W: Worker> {
    /// Implementation for communication channel from Worker to Consumers
    fn respond(&self, id: HandlerId, output: W::Output);
}

/// Link to worker's scope for creating callbacks.
pub struct WorkerLink<W: Worker> {
    scope: WorkerScope<W>,
    responder: Rc<dyn Responder<W>>,
}

impl<W: Worker> WorkerLink<W> {
    /// Create link for a scope.
    pub(crate) fn connect<T>(scope: &WorkerScope<W>, responder: T) -> Self
    where
        T: Responder<W> + 'static,
    {
        WorkerLink {
            scope: scope.clone(),
            responder: Rc::new(responder),
        }
    }

    /// Send response to an worker.
    pub fn respond(&self, id: HandlerId, output: W::Output) {
        self.responder.respond(id, output);
    }

    /// Send a message to the worker
    pub fn send_message<T>(&self, msg: T)
    where
        T: Into<W::Message>,
    {
        self.scope.send(WorkerLifecycleEvent::Message(msg.into()));
    }

    /// Send an input to self
    pub fn send_input<T>(&self, input: T)
    where
        T: Into<W::Input>,
    {
        let handler_id = HandlerId::new(0, false);
        self.scope
            .send(WorkerLifecycleEvent::Input(input.into(), handler_id));
    }

    /// Create a callback which will send a message to the worker when invoked.
    pub fn callback<F, IN, M>(&self, function: F) -> Rc<dyn Fn(IN)>
    where
        M: Into<W::Message>,
        F: Fn(IN) -> M + 'static,
    {
        let scope = self.scope.clone();
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
        let link: WorkerLink<W> = self.clone();
        let js_future = async move {
            let message: W::Message = future.await.into();
            let cb = link.callback(|m: W::Message| m);
            (*cb)(message);
        };
        wasm_bindgen_futures::spawn_local(js_future);
    }
}

impl<W: Worker> fmt::Debug for WorkerLink<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerLink<_>")
    }
}

impl<W: Worker> Clone for WorkerLink<W> {
    fn clone(&self) -> Self {
        WorkerLink {
            scope: self.scope.clone(),
            responder: self.responder.clone(),
        }
    }
}
/// This struct holds a reference to a component and to a global scheduler.
pub(crate) struct WorkerScope<W: Worker> {
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

impl<W: Worker> WorkerScope<W> {
    /// Create worker scope
    pub fn new() -> Self {
        let state = Rc::new(RefCell::new(WorkerState::new()));
        WorkerScope { state }
    }

    /// Schedule message for sending to worker
    pub fn send(&self, event: WorkerLifecycleEvent<W>) {
        let runnable = Box::new(WorkerRunnable {
            state: self.state.clone(),
            event,
        });
        runnable.run();
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
    Create(WorkerLink<W>),
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
                    .expect("worker was not created to process messages")
                    .update(msg);
            }
            WorkerLifecycleEvent::Connected(id) => {
                state
                    .worker
                    .as_mut()
                    .expect("worker was not created to send a connected message")
                    .connected(id);
            }
            WorkerLifecycleEvent::Input(inp, id) => {
                state
                    .worker
                    .as_mut()
                    .expect("worker was not created to process inputs")
                    .handle_input(inp, id);
            }
            WorkerLifecycleEvent::Disconnected(id) => {
                state
                    .worker
                    .as_mut()
                    .expect("worker was not created to send a disconnected message")
                    .disconnected(id);
            }
            WorkerLifecycleEvent::Destroy => {
                let mut worker = state
                    .worker
                    .take()
                    .expect("trying to destroy not existent worker");
                worker.destroy();
                state.destroyed = true;
            }
        }
    }
}
