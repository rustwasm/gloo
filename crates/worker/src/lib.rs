//! Workers are a way to offload tasks to web workers. These are run concurrently using
//! [web-workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).
//!
//! # Communicating with workers
//!
//! ## Bridges
//!
//! After a Worker is spawned, a bridge is created.
//! A Bridge allows bi-directional communication between an worker and a component.
//! Bridges also allow workers to communicate with one another.
//!
//! ## Scopes
//!
//! Scopes are used by workers to communicates with bridges and send updates to itself after
//! a task is finished.
//!
//! # Overhead
//!
//! Workers use web workers. They incur a serialization overhead on the
//! messages they send and receive. Bridges use [bincode](https://!github.com/servo/bincode)
//! to communicate with workers, so the cost is substantially higher
//! than just calling a function.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod bridge;
mod handler_id;
mod messages;
mod native_worker;
mod registrar;
mod scope;
mod spawner;
mod traits;

pub use bridge::Bridge;
pub use handler_id::HandlerId;
pub use registrar::Registrable;
pub use scope::WorkerScope;
pub use spawner::Spawner;
pub use traits::{Spawnable, Worker};

use std::cell::RefCell;
use std::rc::Rc;

/// Alias for `Rc<RefCell<T>>`
pub(crate) type Shared<T> = Rc<RefCell<T>>;

/// Alias for `Rc<dyn Fn(IN)>`
pub(crate) type Callback<IN> = Rc<dyn Fn(IN)>;
