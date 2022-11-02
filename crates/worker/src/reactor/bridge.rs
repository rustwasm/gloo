use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{FusedStream, Stream};
use pinned::mpsc;
use pinned::mpsc::UnboundedReceiver;

use super::messages::{ReactorInput, ReactorOutput};
use super::traits::Reactor;
use super::worker::ReactorWorker;
use crate::actor::WorkerBridge;
use crate::{Codec, WorkerSpawner};

/// A connection manager for components interaction with oneshot workers.
pub struct ReactorBridge<R>
where
    R: Reactor + 'static,
{
    inner: WorkerBridge<ReactorWorker<R>>,
    rx: UnboundedReceiver<<R::OutputStream as Stream>::Item>,
}

impl<R> fmt::Debug for ReactorBridge<R>
where
    R: Reactor,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReactorBridge<_>")
    }
}

impl<R> ReactorBridge<R>
where
    R: Reactor + 'static,
{
    #[inline(always)]
    pub(crate) fn new(
        inner: WorkerBridge<ReactorWorker<R>>,
        rx: UnboundedReceiver<<R::OutputStream as Stream>::Item>,
    ) -> Self {
        Self { inner, rx }
    }

    #[inline(always)]
    pub(crate) fn register_callback<CODEC>(
        spawner: &mut WorkerSpawner<ReactorWorker<R>, CODEC>,
    ) -> UnboundedReceiver<<R::OutputStream as Stream>::Item>
    where
        CODEC: Codec,
    {
        let (tx, rx) = mpsc::unbounded();
        spawner.callback(move |output| match output {
            ReactorOutput::Output(m) => {
                let _ = tx.send_now(m);
            }
            ReactorOutput::Finish => {
                tx.close_now();
            }
        });

        rx
    }

    /// Forks the bridge.
    ///
    /// This method creates a new bridge connected that creates a new reactor on the same worker instance.
    pub fn fork(&self) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let inner = self.inner.fork(Some(move |output| match output {
            ReactorOutput::Output(m) => {
                let _ = tx.send_now(m);
            }
            ReactorOutput::Finish => {
                tx.close_now();
            }
        }));

        Self { inner, rx }
    }

    /// Send a message to the current worker.
    pub fn send(&self, msg: <R::InputStream as Stream>::Item) {
        self.inner.send(ReactorInput::Input(msg));
    }
}

impl<R> Stream for ReactorBridge<R>
where
    R: Reactor + 'static,
{
    type Item = <R::OutputStream as Stream>::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}

impl<R> FusedStream for ReactorBridge<R>
where
    R: Reactor + 'static,
{
    fn is_terminated(&self) -> bool {
        self.rx.is_terminated()
    }
}
