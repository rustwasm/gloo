//! A future-based worker that for each input, one output is produced.
//!
//! ## Example
//!
//! ```rust, no_run
//! use gloo_worker::oneshot::oneshot;
//! use gloo_worker::Spawnable;
//!
//! #[oneshot]
//! async fn Squared(input: u32) -> u32 {
//!     input.pow(2)
//! }
//!
//! # async {
//! // consuming the worker
//! let mut squared_bridge = Squared::spawner().spawn("...");
//! assert_eq!(squared_bridge.run(2).await, 4);
//! # };
//! ```

mod bridge;
mod registrar;
mod spawner;
mod traits;
mod worker;

pub use bridge::OneshotBridge;
pub use registrar::OneshotRegistrar;
pub use spawner::OneshotSpawner;
pub use traits::Oneshot;

/// Creates an oneshot worker.
///
/// See [module level documentation](self) for more information.
#[doc(inline)]
#[cfg(feature = "futures")]
pub use gloo_worker_macros::oneshot;
