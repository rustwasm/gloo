use std::fmt;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::bridge::WorkerBridge;
use crate::Worker;

/// Worker Kind
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum WorkerKind {
    /// A dedicated Worker.
    Dedicated,
}

/// A spawner to create workers.
#[derive(Clone)]
pub struct WorkerSpawner<W>
where
    W: Worker,
{
    kind: WorkerKind,
    _marker: PhantomData<W>,
    callback: Option<Rc<dyn Fn(W::Output)>>,
}

impl<W: Worker> fmt::Debug for WorkerSpawner<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerScope<_>")
    }
}

impl<W> WorkerSpawner<W>
where
    W: Worker,
{
    pub fn new(kind: WorkerKind) -> Self {
        Self {
            kind,
            _marker: PhantomData,
            callback: None,
        }
    }

    pub fn callback<F>(&mut self, cb: F) -> &mut Self
    where
        F: 'static + Fn(W::Output),
    {
        self.callback = Some(Rc::new(cb));

        self
    }

    pub fn spawn(&self, path: &str) -> WorkerBridge<W> {
        todo!()
    }
}
