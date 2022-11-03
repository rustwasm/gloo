use std::fmt;
use std::pin::Pin;

use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};

/// A stream used by reactors and reactor bridges.
pub struct ReactorStream<T> {
    rx: Pin<Box<dyn FusedStream<Item = T>>>,
}

impl<T> fmt::Debug for ReactorStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReactorSource<_>").finish()
    }
}

impl<T> Stream for ReactorStream<T> {
    type Item = T;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_next(cx)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.rx.size_hint()
    }
}

impl<T> FusedStream for ReactorStream<T> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.rx.is_terminated()
    }
}

/// A helper trait to extract the input type from a [ReactorStream].
pub trait ReactorConsumable: Stream + FusedStream {
    /// Creates a ReactorReceiver.
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = <Self as Stream>::Item> + FusedStream + 'static;
}

impl<I> ReactorConsumable for ReactorStream<I> {
    #[inline]
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = <Self as Stream>::Item> + FusedStream + 'static,
    {
        Self {
            rx: Box::pin(stream),
        }
    }
}
