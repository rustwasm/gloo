//! A future-based worker that for each input, one output is produced.

mod bridge;
mod registrar;
mod spawner;
mod traits;
mod worker;

pub use bridge::OneshotBridge;
pub use registrar::OneshotRegistrar;
pub use spawner::OneshotSpawner;
pub use traits::Oneshot;
