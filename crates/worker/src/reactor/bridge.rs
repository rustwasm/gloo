use std::cell::Cell;
use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::stream::{FusedStream, Stream};
use pinned::mpsc;
use pinned::mpsc::{UnboundedReceiver, UnboundedSender};

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

    pub(crate) fn output_callback(
        tx: &UnboundedSender<<R::OutputStream as Stream>::Item>,
        output: ReactorOutput<<R::OutputStream as Stream>::Item>,
    ) {
        match output {
            ReactorOutput::Output(m) => {
                let _ = tx.send_now(m);
            }
            ReactorOutput::Finish => {
                tx.close_now();
            }
        }
    }

    #[inline(always)]
    pub(crate) fn register_callback<CODEC>(
        spawner: &mut WorkerSpawner<ReactorWorker<R>, CODEC>,
    ) -> UnboundedReceiver<<R::OutputStream as Stream>::Item>
    where
        CODEC: Codec,
    {
        let (tx, rx) = mpsc::unbounded();
        spawner.callback(move |output| Self::output_callback(&tx, output));

        rx
    }

    /// Forks the bridge.
    ///
    /// This method creates a new bridge connected to a new reactor on the same worker instance.
    pub fn fork(&self) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let inner = self
            .inner
            .fork(Some(move |output| Self::output_callback(&tx, output)));

        Self { inner, rx }
    }

    /// Sends a message to the current worker.
    pub fn send(&self, msg: <R::InputStream as Stream>::Item) {
        self.inner.send(ReactorInput::Input(msg));
    }

    /// Splits the bridge into a sender - receiver pair.
    pub fn split(self) -> (ReactorBridgeSender<R>, ReactorBridgeReceiver<R>) {
        thread_local! {
            static PAIR_ID: Cell<usize> = Cell::new(0);
        }
        let id = PAIR_ID.with(|m| {
            let id = m.get();
            m.set(id + 1);
            id
        });

        (
            ReactorBridgeSender {
                id,
                inner: self.inner,
            },
            ReactorBridgeReceiver { id, rx: self.rx },
        )
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

/// The receiver half of a [ReactorBridge].
pub struct ReactorBridgeReceiver<R>
where
    R: Reactor + 'static,
{
    id: usize,
    rx: UnboundedReceiver<<R::OutputStream as Stream>::Item>,
}

impl<R> fmt::Debug for ReactorBridgeReceiver<R>
where
    R: Reactor,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReactorBridgeReceiver<_>")
    }
}

impl<R> Stream for ReactorBridgeReceiver<R>
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

impl<R> FusedStream for ReactorBridgeReceiver<R>
where
    R: Reactor + 'static,
{
    fn is_terminated(&self) -> bool {
        self.rx.is_terminated()
    }
}

impl<R> ReactorBridgeReceiver<R>
where
    R: Reactor + 'static,
{
    /// Checks if this ReactorBridgeReceiver and ReactorBridgeSender are from the same bridge.
    pub fn is_pair_of(&self, other: &ReactorBridgeSender<R>) -> bool {
        self.id == other.id
    }
}

/// The sender half of a [ReactorBridge].
pub struct ReactorBridgeSender<R>
where
    R: Reactor + 'static,
{
    id: usize,
    inner: WorkerBridge<ReactorWorker<R>>,
}

impl<R> fmt::Debug for ReactorBridgeSender<R>
where
    R: Reactor,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ReactorBridgeSender<_>")
    }
}

impl<R> ReactorBridgeSender<R>
where
    R: Reactor + 'static,
{
    /// Send a message to the current worker.
    pub fn send(&self, msg: <R::InputStream as Stream>::Item) {
        self.inner.send(ReactorInput::Input(msg));
    }

    /// Checks if this ReactorBridgeSender and ReactorBridgeReceiver are from the same bridge.
    pub fn is_pair_of(&self, other: &ReactorBridgeReceiver<R>) -> bool {
        self.id == other.id
    }

    /// Consumes the sender and receiver, returns the original bridge.
    pub fn unsplit(self, rx: ReactorBridgeReceiver<R>) -> ReactorBridge<R> {
        assert!(
            self.is_pair_of(&rx),
            "not the same pair of receiver and sender."
        );

        ReactorBridge {
            inner: self.inner,
            rx: rx.rx,
        }
    }

    /// Forks the bridge.
    ///
    /// This method creates a new bridge sender and receiver pair connected to a new reactor on the same worker instance.
    pub fn fork(&self) -> (ReactorBridgeSender<R>, ReactorBridgeReceiver<R>) {
        let (tx, rx) = mpsc::unbounded();
        let inner = self.inner.fork(Some(move |output| {
            ReactorBridge::<R>::output_callback(&tx, output)
        }));

        ReactorBridge { inner, rx }.split()
    }
}
