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
mod messages;
mod pool;
mod registrar;
mod scope;
mod spawner;
mod worker;
mod worker_ext;

pub use registrar::WorkerRegistrar;
pub use spawner::{WorkerKind, WorkerSpawner};

pub(crate) use pool::*;
pub(crate) use scope::{WorkerLifecycleEvent, WorkerScope};
use std::cell::RefCell;
pub use worker::{Private, PrivateWorker, Public, PublicWorker};

use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Alias for `Rc<RefCell<T>>`
pub type Shared<T> = Rc<RefCell<T>>;

/// Alias for `Rc<dyn Fn(IN)>`
pub type Callback<IN> = Rc<dyn Fn(IN)>;

/// Declares the behavior of the worker.
pub trait Worker: Sized + 'static {
    /// Reach capability of the worker.
    type Reach: Discoverer<Worker = Self>;
    /// Type of an input message.
    type Message;
    /// Incoming message type.
    type Input: Serialize + for<'de> Deserialize<'de>;
    /// Outgoing message type.
    type Output: Serialize + for<'de> Deserialize<'de>;

    /// Creates an instance of an worker.
    fn create(link: WorkerScope<Self>) -> Self;

    /// This method called on every update message.
    fn update(&mut self, msg: Self::Message);

    /// This method called on when a new bridge created.
    fn connected(&mut self, _id: HandlerId) {}

    /// This method called on every incoming message.
    fn handle_input(&mut self, msg: Self::Input, id: HandlerId);

    /// This method called on when a new bridge destroyed.
    fn disconnected(&mut self, _id: HandlerId) {}

    /// This method called when the worker is destroyed.
    fn destroy(&mut self) {}

    /// Represents the name of loading resource for remote workers which
    /// have to live in a separate files.
    fn name_of_resource() -> &'static str {
        "main.js"
    }

    /// Indicates whether the name of the resource is relative.
    ///
    /// The default implementation returns `false`, which will cause the result
    /// returned by [`Self::name_of_resource`] to be interpreted as an absolute
    /// URL. If `true` is returned, it will be interpreted as a relative URL.
    fn resource_path_is_relative() -> bool {
        false
    }

    /// Signifies if resource is a module.
    /// This has pending browser support.
    fn is_module() -> bool {
        false
    }
}

/// Id of responses handler.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Copy)]
pub struct HandlerId(usize, bool);

impl HandlerId {
    fn new(id: usize, respondable: bool) -> Self {
        HandlerId(id, respondable)
    }
    fn raw_id(self) -> usize {
        self.0
    }
    /// Indicates if a handler id corresponds to callback in the Worker runtime.
    pub fn is_respondable(self) -> bool {
        self.1
    }
}

/// Determine a visibility of an worker.
#[doc(hidden)]
pub trait Discoverer {
    type Worker: Worker;

    /// Spawns an worker and returns `Bridge` implementation.
    fn spawn_or_join(
        _callback: Option<Callback<<Self::Worker as Worker>::Output>>,
    ) -> Box<dyn Bridge<Self::Worker>>;
}

/// Bridge to a specific kind of worker.
pub trait Bridge<W: Worker> {
    /// Send a message to an worker.
    fn send(&mut self, msg: W::Input);
}

/// This trait allows registering or getting the address of a worker.
pub trait Bridged: Worker + Sized + 'static {
    /// Creates a messaging bridge between a worker and the component.
    fn bridge(callback: Callback<Self::Output>) -> Box<dyn Bridge<Self>>;
}

impl<T> Bridged for T
where
    T: Worker,
    <T as Worker>::Reach: Discoverer<Worker = T>,
{
    fn bridge(callback: Callback<Self::Output>) -> Box<dyn Bridge<Self>> {
        Self::Reach::spawn_or_join(Some(callback))
    }
}
