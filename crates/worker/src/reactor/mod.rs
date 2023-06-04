//! A future-based worker that can consume many inputs and produce many outputs.
//!
//! ## Example
//!
//! ```rust, no_run
//! use gloo_worker::reactor::{reactor, ReactorScope};
//! use gloo_worker::Spawnable;
//! use futures::{sink::SinkExt, StreamExt};
//!
//! #[reactor]
//! async fn SquaredOnDemand(mut scope: ReactorScope<u64, u64>) {
//!     while let Some(m) = scope.next().await {
//!         if scope.send(m.pow(2)).await.is_err() {
//!             break;
//!         }
//!     }
//! }
//! # async {
//! let mut bridge = SquaredOnDemand::spawner().spawn("...");
//!
//! bridge.send_input(2);
//!
//! assert_eq!(bridge.next().await, Some(4));
//! assert_eq!(bridge.next().await, None);
//! # };
//! ```

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

/// Creates a reactor worker.
///
/// See [module level documentation](self) for more information.
#[doc(inline)]
#[cfg(feature = "futures")]
pub use gloo_worker_macros::reactor;
