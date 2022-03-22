use wasm_bindgen::prelude::*;

use crate::handler_id::HandlerId;
use crate::scope::WorkerScope;
use crate::traits::Worker;
use crate::Shared;

pub(crate) struct WorkerState<W> {
    worker: Option<W>,
    // TODO: Use worker field to control create message this flag
    destroyed: bool,
}

impl<W> WorkerState<W> {
    pub fn new() -> Self {
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
    Destroy(WorkerScope<W>),
}

pub(crate) struct WorkerRunnable<W: Worker> {
    pub state: Shared<WorkerState<W>>,
    pub event: WorkerLifecycleEvent<W>,
}

impl<W> WorkerRunnable<W>
where
    W: Worker,
{
    pub fn run(self) {
        let mut state = self.state.borrow_mut();

        // We should block all event other than message after a worker is destroyed.
        match self.event {
            WorkerLifecycleEvent::Create(link) => {
                if state.destroyed {
                    return;
                }
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
                if state.destroyed {
                    return;
                }

                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to send a connected message")
                    .connected(id);
            }
            WorkerLifecycleEvent::Input(inp, id) => {
                if state.destroyed {
                    return;
                }
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to process inputs")
                    .received(inp, id);
            }
            WorkerLifecycleEvent::Disconnected(id) => {
                if state.destroyed {
                    return;
                }
                state
                    .worker
                    .as_mut()
                    .expect_throw("worker was not created to send a disconnected message")
                    .disconnected(id);
            }
            WorkerLifecycleEvent::Destroy(scope) => {
                if state.destroyed {
                    return;
                }
                let mut worker = state
                    .worker
                    .take()
                    .expect_throw("trying to destroy not existent worker");
                let should_terminate_now = worker.destroy();

                scope.set_closable();

                if should_terminate_now {
                    scope.close();
                }
                state.destroyed = true;
            }
        }
    }
}
