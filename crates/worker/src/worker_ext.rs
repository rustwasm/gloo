use js_sys::Uint8Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

use crate::messages::Packed;

pub(crate) fn worker_self() -> DedicatedWorkerGlobalScope {
    JsValue::from(js_sys::global()).into()
}

pub(crate) trait NativeWorkerExt {
    fn set_on_packed_message<T>(&self, handler: impl 'static + Fn(T))
    where
        T: Packed;

    fn post_packed_message<T>(&self, data: T)
    where
        T: Packed;
}

macro_rules! worker_ext_impl {
    ($($type:path),+) => {$(
        impl NativeWorkerExt for $type {
            fn set_on_packed_message<T>(&self, handler: impl 'static + Fn(T))
            where
                T: Packed
            {
                let handler = move |message: MessageEvent| {
                    let data = Uint8Array::from(message.data()).to_vec();
                    let msg = T::unpack(&data);
                    handler(msg);
                };
                let closure = Closure::wrap(Box::new(handler) as Box<dyn Fn(MessageEvent)>);
                self.set_onmessage(Some(closure.as_ref().unchecked_ref()));
                // Memory leak?
                closure.forget();
            }

            fn post_packed_message<T>(&self, data: T)
            where
                T: Packed
            {
                self.post_message(&Uint8Array::from(data.pack().as_slice()))
                    .expect_throw("failed to post message");
            }
        }
    )+};
}

worker_ext_impl! {
    web_sys::Worker, DedicatedWorkerGlobalScope
}
