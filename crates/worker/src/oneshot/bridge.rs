use futures::stream::StreamExt;
use pinned::mpsc;
use pinned::mpsc::UnboundedReceiver;

use super::traits::Oneshot;
use super::worker::OneshotWorker;
use crate::actor::WorkerBridge;
use crate::{Codec, WorkerSpawner};

/// A connection manager for components interaction with oneshot workers.
#[derive(Debug)]
pub struct OneshotBridge<N>
where
    N: Oneshot + 'static,
{
    inner: WorkerBridge<OneshotWorker<N>>,
    rx: UnboundedReceiver<N::Output>,
}

impl<N> OneshotBridge<N>
where
    N: Oneshot + 'static,
{
    pub(crate) fn new(
        inner: WorkerBridge<OneshotWorker<N>>,
        rx: UnboundedReceiver<N::Output>,
    ) -> Self {
        Self { inner, rx }
    }

    pub(crate) fn register_callback<CODEC>(
        spawner: &mut WorkerSpawner<OneshotWorker<N>, CODEC>,
    ) -> UnboundedReceiver<N::Output>
    where
        CODEC: Codec,
    {
        let (tx, rx) = mpsc::unbounded();
        spawner.callback(move |output| {
            let _ = tx.send_now(output);
        });

        rx
    }

    /// Forks the bridge.
    ///
    /// This method creates a new bridge that can be used to execute tasks on the same worker instance.
    pub fn fork(&self) -> Self {
        let (tx, rx) = mpsc::unbounded();
        let inner = self.inner.fork(Some(move |output| {
            let _ = tx.send_now(output);
        }));

        Self { inner, rx }
    }

    /// Run the the current oneshot worker once in the current worker instance.
    pub async fn run(&mut self, input: N::Input) -> N::Output {
        self.inner.send(input);

        self.rx
            .next()
            .await
            .expect("failed to receive result from worker")
    }
}
