use std::future::Future;

use serde::{Deserialize, Serialize};

/// A future-based worker that for each input, one output is produced.
pub trait Oneshot {
    /// Incoming message type.
    type Input: Serialize + for<'de> Deserialize<'de>;
    /// Outgoing message type.
    type Output: Serialize + for<'de> Deserialize<'de>;

    /// Future type created for current task.
    type Future: 'static + Future<Output = Self::Output>;

    /// Runs a oneshot worker.
    fn run(input: Self::Input) -> Self::Future;
}
