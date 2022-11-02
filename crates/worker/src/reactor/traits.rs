use futures::Stream;

use super::ReactorConsumable;

/// A reactor worker.
pub trait Reactor {
    /// The Reactor Receiver.
    type InputStream: ReactorConsumable;
    /// The Reactor OutputStream.
    type OutputStream: Stream;

    /// Runs a reactor worker.
    fn spawn(inputs: Self::InputStream) -> Self::OutputStream;
}
