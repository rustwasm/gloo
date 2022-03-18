use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

use crate::messages::{FromWorker, Packed, ToWorker};
use crate::traits::Worker;

pub(crate) fn worker_self() -> DedicatedWorkerGlobalScope {
    JsValue::from(js_sys::global()).into()
}

pub(crate) trait WorkerExt {
    fn set_onmessage_closure(&self, handler: impl 'static + Fn(Vec<u8>));

    fn post_message_vec(&self, data: Vec<u8>);

    fn post_to_worker<W>(&self, to_worker: ToWorker<W>)
    where
        W: Worker;

    fn post_from_worker<W>(&self, from_worker: FromWorker<W>)
    where
        W: Worker;
}

macro_rules! worker_ext_impl {
    ($($type:path),+) => {$(
        impl WorkerExt for $type {
            fn set_onmessage_closure(&self, handler: impl 'static + Fn(Vec<u8>)) {
                let handler = move |message: MessageEvent| {
                    let data = Uint8Array::from(message.data()).to_vec();
                    handler(data);
                };
                let closure = Closure::wrap(Box::new(handler) as Box<dyn Fn(MessageEvent)>);
                self.set_onmessage(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            }

            fn post_message_vec(&self, data: Vec<u8>) {
                self.post_message(&Uint8Array::from(data.as_slice()))
                    .expect("failed to post message");
            }

            fn post_to_worker<W>(&self, to_worker: ToWorker<W>)
            where
                W: Worker
            {
                let msg = to_worker.pack();
                self.post_message_vec(msg);
            }

            fn post_from_worker<W>(&self, from_worker: FromWorker<W>)
            where
                W: Worker
            {
                let msg = from_worker.pack();
                self.post_message_vec(msg);
            }
        }
    )+};
}

worker_ext_impl! {
    web_sys::Worker, DedicatedWorkerGlobalScope
}
