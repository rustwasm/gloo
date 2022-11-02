//! A future-based worker that can consume many inputs and produce many outputs.

mod messages;
mod receiver;
mod traits;
mod worker;

pub use receiver::{ReactorConsumable, ReactorReceiver};
pub use traits::Reactor;
