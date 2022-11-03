//! A future-based worker that can consume many inputs and produce many outputs.

mod bridge;
mod messages;
mod registrar;
mod source;
mod spawner;
mod traits;
mod worker;

pub use bridge::ReactorBridge;
pub use registrar::ReactorRegistrar;
pub use source::{ReactorConsumable, ReactorStream};
pub use spawner::ReactorSpawner;
pub use traits::Reactor;
