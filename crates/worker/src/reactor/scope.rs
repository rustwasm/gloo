use std::convert::Infallible;
use std::fmt;
use std::pin::Pin;

use futures::stream::{FusedStream, Stream};
use futures::task::{Context, Poll};
use futures::Sink;

/// A handle to communicate with bridges.
pub struct ReactorScope<I, O> {
    from_bridge: Pin<Box<dyn FusedStream<Item = I>>>,
    to_bridge: Pin<Box<dyn Sink<O, Error = Infallible>>>,
}

impl<I, O> fmt::Debug for ReactorScope<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReactorScope<_>").finish()
    }
}

impl<I, O> Stream for ReactorScope<I, O> {
    type Item = I;

    #[inline(always)]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.from_bridge).poll_next(cx)
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.from_bridge.size_hint()
    }
}

impl<I, O> FusedStream for ReactorScope<I, O> {
    #[inline(always)]
    fn is_terminated(&self) -> bool {
        self.from_bridge.is_terminated()
    }
}

/// A helper trait to extract the input and output type from a [ReactorStream].
pub trait ReactorScoped: Stream + FusedStream {
    /// The Input Message.
    type Input;
    /// The Output Message.
    type Output;

    /// Creates a ReactorReceiver.
    fn new<IS, OS>(from_bridge: IS, to_bridge: OS) -> Self
    where
        IS: Stream<Item = Self::Input> + FusedStream + 'static,
        OS: Sink<Self::Output, Error = Infallible> + 'static;
}

impl<I, O> ReactorScoped for ReactorScope<I, O> {
    type Input = I;
    type Output = O;

    #[inline]
    fn new<IS, OS>(from_bridge: IS, to_bridge: OS) -> Self
    where
        IS: Stream<Item = Self::Input> + FusedStream + 'static,
        OS: Sink<Self::Output, Error = Infallible> + 'static,
    {
        Self {
            from_bridge: Box::pin(from_bridge),
            to_bridge: Box::pin(to_bridge),
        }
    }
}

impl<I, O> Sink<O> for ReactorScope<I, O> {
    type Error = Infallible;

    fn start_send(mut self: Pin<&mut Self>, item: O) -> Result<(), Self::Error> {
        Pin::new(&mut self.to_bridge).start_send(item)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.to_bridge).poll_close(cx)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.to_bridge).poll_flush(cx)
    }

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.to_bridge).poll_flush(cx)
    }
}
