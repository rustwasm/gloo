//! Workers are a way to offload tasks to web workers. These are run concurrently using
//! [web-workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Using_web_workers).
//!
//! # Types of Workers
//!
//! ## Reaches
//!
//! * Public - There will exist at most one instance of a Public Worker at any given time.
//!   Bridges will spawn or connect to an already spawned worker in a web worker.
//!   When no bridges are connected to this worker, the worker will disappear.
//!
//! * Private - Spawn a new worker in a web worker for every new bridge. This is good for
//!   moving shared but independent behavior that communicates with the browser out of components.
//!   When the the connected bridge is dropped, the worker will disappear.
//!
//! # Communicating with workers
//!
//! ## Bridges
//!
//! A bridge allows bi-directional communication between an worker and a component.
//! Bridges also allow workers to communicate with one another.
//!
//! ## Dispatchers
//!
//! A dispatcher allows uni-directional communication between a component and an worker.
//! A dispatcher allows a component to send messages to an worker.
//!
//! # Overhead
//!
//! Workers use web workers (i.e. Private and Public). They incur a serialization overhead on the
//! messages they send and receive. Workers use [bincode](https://!github.com/servo/bincode)
//! to communicate with other browser worker, so the cost is substantially higher
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
