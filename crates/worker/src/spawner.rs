use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

use js_sys::Array;
use web_sys::{Blob, BlobPropertyBag, Url};

use crate::bridge::{CallbackMap, WorkerBridge};
use crate::handler_id::HandlerId;
use crate::messages::{FromWorker, Packed};
use crate::worker_ext::WorkerExt;
use crate::Worker;

fn create_worker(path: &str) -> web_sys::Worker {
    let wasm_url = path.replace(".js", "_bg.wasm");
    let array = Array::new();
    array.push(&format!(r#"importScripts("{}");wasm_bindgen("{}");"#, path, wasm_url).into());
    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("application/javascript"),
    )
    .unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    web_sys::Worker::new(&url).expect("failed to spawn worker")
}

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
    _kind: WorkerKind,
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
            _kind: kind,
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
        let pending_queue = Rc::new(RefCell::new(Some(Vec::new())));
        let callbacks: Rc<RefCell<CallbackMap<W>>> = Rc::default();

        let handler = {
            let pending_queue = pending_queue.clone();
            let callbacks = callbacks.clone();

            move |data: Vec<u8>, worker: &web_sys::Worker| {
                let msg = FromWorker::<W>::unpack(&data);
                match msg {
                    FromWorker::WorkerLoaded => {
                        if let Some(pending_queue) = pending_queue.borrow_mut().take() {
                            for to_worker in pending_queue.into_iter() {
                                worker.post_to_worker(to_worker);
                            }
                        }
                    }
                    FromWorker::ProcessOutput(id, output) => {
                        let mut callbacks = callbacks.borrow_mut();

                        if let Some(m) = callbacks.get(&id) {
                            if let Some(m) = Weak::upgrade(m) {
                                m(output);
                            } else {
                                callbacks.remove(&id);
                            }
                        }
                    }
                }
            }
        };

        let handler_cell = Rc::new(RefCell::new(Some(handler)));

        let worker = {
            let handler_cell = handler_cell.clone();
            let worker = create_worker(path);
            let worker_clone = worker.clone();
            worker.set_onmessage_closure(move |data: Vec<u8>| {
                if let Some(handler) = handler_cell.borrow().as_ref() {
                    handler(data, &worker_clone)
                }
            });
            worker
        };

        WorkerBridge::<W>::new(
            HandlerId::new_inc(),
            worker,
            pending_queue,
            callbacks,
            self.callback.clone(),
        )
    }
}
