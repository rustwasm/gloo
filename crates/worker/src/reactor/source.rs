use std::fmt;
use std::pin::Pin;

use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};

/// A stream that produces inputs for a reactor.
pub struct ReactorSource<I> {
    rx: Pin<Box<dyn FusedStream<Item = I>>>,
}

impl<I> fmt::Debug for ReactorSource<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReactorSource<_>").finish()
    }
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
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = <Self as Stream>::Item> + FusedStream + 'static;
}

impl<I> ReactorConsumable for ReactorSource<I> {
    fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = <Self as Stream>::Item> + FusedStream + 'static,
    {
        Self {
            rx: Box::pin(stream),
        }
    }
}
