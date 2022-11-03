use std::collections::HashMap;

use futures::stream::{Stream, StreamExt};
use pinned::mpsc;
use pinned::mpsc::UnboundedSender;

use super::messages::{ReactorInput, ReactorOutput};
use super::source::ReactorConsumable;
use super::traits::Reactor;
use crate::actor::{HandlerId, Worker, WorkerDestroyHandle, WorkerScope};

pub(crate) enum Message {
    ReactorExited(HandlerId),
}

pub(crate) struct ReactorWorker<R>
where
    R: 'static + Reactor,
{
    senders: HashMap<HandlerId, UnboundedSender<<R::InputStream as Stream>::Item>>,
    destruct_handle: Option<WorkerDestroyHandle<Self>>,
}

impl<R> Worker for ReactorWorker<R>
where
    R: 'static + Reactor,
{
    type Input = ReactorInput<<R::InputStream as Stream>::Item>;
    type Message = Message;
    type Output = ReactorOutput<<R::OutputStream as Stream>::Item>;

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {
            senders: HashMap::new(),
            destruct_handle: None,
        }
    }

    fn update(&mut self, scope: &WorkerScope<Self>, msg: Self::Message) {
        match msg {
            Self::Message::ReactorExited(id) => {
                scope.respond(id, ReactorOutput::Finish);
                self.senders.remove(&id);
            }
        }

        // All reactors have closed themselves, the worker can now close.
        if self.destruct_handle.is_some() && self.senders.is_empty() {
            self.destruct_handle = None;
        }
    }

    fn connected(&mut self, scope: &WorkerScope<Self>, id: HandlerId) {
        let consumer = {
            let (tx, rx) = mpsc::unbounded();
            self.senders.insert(id, tx);
            R::InputStream::new(rx)
        };

        let producer = R::create(consumer);

        let scope_clone = scope.clone();
        scope.send_future(async move {
            futures::pin_mut!(producer);

            while let Some(m) = producer.next().await {
                scope_clone.respond(id, ReactorOutput::Output(m));
            }

            Message::ReactorExited(id)
        });
    }

    fn received(&mut self, _scope: &WorkerScope<Self>, input: Self::Input, id: HandlerId) {
        match input {
            Self::Input::Input(input) => {
                if let Some(m) = self.senders.get_mut(&id) {
                    let _result = m.send_now(input);
                }
            }
        }
    }

    fn disconnected(&mut self, _scope: &WorkerScope<Self>, id: HandlerId) {
        // We close this channel, but drop it when the reactor has exited itself.
        if let Some(m) = self.senders.get_mut(&id) {
            m.close_now();
        }
    }

    fn destroy(&mut self, _scope: &WorkerScope<Self>, destruct: WorkerDestroyHandle<Self>) {
        if !self.senders.is_empty() {
            self.destruct_handle = Some(destruct);
        }
    }
}
