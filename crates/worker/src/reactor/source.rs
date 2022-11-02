use std::pin::Pin;

use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};
use pinned::mpsc;

/// A stream that produces inputs for a reactor.
#[derive(Debug)]
pub struct ReactorSource<I> {
    rx: mpsc::UnboundedReceiver<I>,
}

impl<I> Stream for ReactorSource<I> {
    type Item = I;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}

impl<I> FusedStream for ReactorSource<I> {
    #[inline]
    fn is_terminated(&self) -> bool {
        self.rx.is_terminated()
    }
}

/// A trait to extract input type from [ReactorSource].
pub trait ReactorConsumable: Stream + FusedStream {
    /// Creates a ReactorReceiver.
    fn new(rx: mpsc::UnboundedReceiver<Self::Item>) -> Self;
}

impl<I> ReactorConsumable for ReactorSource<I> {
    fn new(rx: mpsc::UnboundedReceiver<I>) -> Self {
        Self { rx }
    }
}
