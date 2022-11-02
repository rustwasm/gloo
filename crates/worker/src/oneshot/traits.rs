use std::future::Future;

/// A future-based worker that for each input, one output is produced.
pub trait Oneshot {
    /// Incoming message type.
    type Input;
    /// Outgoing message type.
    type Output;

    /// Future type created for current task.
    type Future: 'static + Future<Output = Self::Output>;

    /// Runs a oneshot worker.
    fn run(input: Self::Input) -> Self::Future;
}
