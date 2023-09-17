//! Workers are a way to offload tasks to web workers. These are run concurrently using
//! [web-workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).
//!
//! # Communicating with workers
//!
//! ### Bridges
//!
//! After a Worker is spawned, a bridge is created.
//! A Bridge allows bi-directional communication between an worker and a component.
//! Bridges also allow workers to communicate with one another.
//!
//! ### Scopes
//!
//! Scopes are used by workers to communicates with bridges and send updates to itself after
//! a task is finished.
//!
//! ### Overhead
//!
//! Gloo Workers use web workers. They incur a serialization overhead on the
//! messages they send and receive. Bridges use [bincode](https://github.com/servo/bincode)
//! by default to communicate with workers, so the cost is substantially higher
//! than just calling a function.
//!
//! # API
//!
//! The API is exposed in two different ways.
//! 1. Using the `Worker` trait.
//! 2. Using the `#[oneshot]` and `#[reactor]` macros.
//!
//! ## Worker trait
//!
//! The [`Worker`] trait is the core of the API. It allows you to spawn workers and communicate
//! with them. It provides an actor model to communicate with for workers.
//!
//! See the [`Worker`] trait for more information.
//!
//! ## Macros
//!
//! The macros provide a function-like syntax to spawn workers and communicate with them.
//! There are two macros:
//! 1. [`#[oneshot]`](oneshot) - Worker where each input produces a single output.
//! 2. [`#[reactor]`](reactor) - Worker that receives input(s) and may produce output(s).

#![deny(
    clippy::all,
    missing_docs,
    missing_debug_implementations,
    bare_trait_objects,
    anonymous_parameters,
    elided_lifetimes_in_paths
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod actor;
mod codec;
#[cfg(feature = "futures")]
pub mod oneshot;
#[cfg(feature = "futures")]
pub mod reactor;
mod traits;

pub use actor::*;
pub use codec::{Bincode, Codec};
pub use traits::*;
