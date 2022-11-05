//! A future-based worker that can consume many inputs and produce many outputs.

mod bridge;
mod messages;
mod registrar;
mod scope;
mod spawner;
mod traits;
mod worker;

pub use bridge::{ReactorBridge, ReactorBridgeSinkError};
pub use registrar::ReactorRegistrar;
pub use scope::{ReactorScope, ReactorScoped};
pub use spawner::ReactorSpawner;
pub use traits::Reactor;
