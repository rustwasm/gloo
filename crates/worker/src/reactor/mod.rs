//! A future-based worker that can consume many inputs and produce many outputs.

mod bridge;
mod messages;
mod receiver;
mod registrar;
mod spawner;
mod traits;
mod worker;

pub use bridge::ReactorBridge;
pub use receiver::{ReactorConsumable, ReactorSource};
pub use registrar::ReactorRegistrar;
pub use spawner::ReactorSpawner;
pub use traits::Reactor;
