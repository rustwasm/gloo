use std::cell::RefCell;
use std::fmt;
#[cfg(feature = "futures")]
use std::future::Future;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use wasm_bindgen_futures::spawn_local;

use crate::codec::Codec;
use crate::handler_id::HandlerId;
use crate::lifecycle::{WorkerLifecycleEvent, WorkerRunnable, WorkerState};
use crate::messages::FromWorker;
use crate::native_worker::{DedicatedWorker, NativeWorkerExt, WorkerSelf};
use crate::traits::Worker;
use crate::Shared;

/// This struct holds a reference to a component and to a global scheduler.
pub struct WorkerScope<W: Worker> {
    state: Shared<WorkerState<W>>,
    closable: Rc<AtomicBool>,
    post_msg: Rc<dyn Fn(FromWorker<W>)>,
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
            closable: self.closable.clone(),
            post_msg: self.post_msg.clone(),
        }
    }
}

impl<W> WorkerScope<W>
where
    W: Worker,
{
    /// Create worker scope
    pub(crate) fn new<CODEC>() -> Self
    where
        CODEC: Codec,
    {
        let post_msg = move |msg: FromWorker<W>| {
            DedicatedWorker::worker_self().post_packed_message::<_, CODEC>(msg)
        };

        let state = Rc::new(RefCell::new(WorkerState::new()));
        WorkerScope {
            post_msg: Rc::new(post_msg),
            state,
            closable: AtomicBool::new(false).into(),
        }
    }

    /// Schedule message for sending to worker
    pub(crate) fn send(&self, event: WorkerLifecycleEvent<W>) {
        let state = self.state.clone();

        // We can implement a custom scheduler,
        // but it's easier to borrow the one from wasm-bindgen-futures.
        spawn_local(async move {
            WorkerRunnable { state, event }.run();
        });
    }

    /// Send response to a worker bridge.
    pub fn respond(&self, id: HandlerId, output: W::Output) {
        let msg = FromWorker::<W>::ProcessOutput(id, output);
        (self.post_msg)(msg);
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

    /// Notifies the scope that close can be called.
    pub(crate) fn set_closable(&self) {
        self.closable.store(true, Ordering::Relaxed);
    }

    /// Closes the current worker.
    ///
    /// Note: You can only call this method after the `destroy` lifecycle event is notified.
    ///
    /// # Panics
    ///
    /// This method would panic if it is called before the `destroy` lifecycle event.
    pub fn close(&self) {
        assert!(
            self.closable.load(Ordering::Relaxed),
            "a worker can only be closed after its destroy method is notified."
        );

        self.send(WorkerLifecycleEvent::Destroy);
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
        let scope = self.clone();

        let closure = move |input: IN| {
            let future: FU = function(input);
            scope.send_future(future);
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
        let scope = self.clone();
        let js_future = async move {
            let message: W::Message = future.await.into();
            let cb = scope.callback(|m: W::Message| m);
            (*cb)(message);
        };
        wasm_bindgen_futures::spawn_local(js_future);
    }
}