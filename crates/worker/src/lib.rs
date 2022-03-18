//! Workers are a way to offload tasks to web workers. These are run concurrently using
//! [web-workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).
//!
//! # Communicating with workers
//!
//! ## Bridges
//!
//! A bridge allows bi-directional communication between an worker and a component.
//! Bridges also allow workers to communicate with one another.
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
mod registrar;
mod scope;
mod spawner;
mod traits;
mod worker_ext;

pub use handler_id::HandlerId;
pub use registrar::WorkerRegistrar;
pub use scope::WorkerScope;
pub use spawner::WorkerSpawner;
pub use traits::Worker;

use std::cell::RefCell;
use std::rc::Rc;

/// Alias for `Rc<RefCell<T>>`
pub type Shared<T> = Rc<RefCell<T>>;

/// Alias for `Rc<dyn Fn(IN)>`
pub type Callback<IN> = Rc<dyn Fn(IN)>;
