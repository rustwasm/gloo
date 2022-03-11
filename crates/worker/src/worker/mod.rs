mod private;
mod public;
mod queue;

pub use private::{Private, PrivateWorker};
pub use public::{Public, PublicWorker};

use crate::{HandlerId, Responder, Worker};
use js_sys::{Array, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{
    Blob, BlobPropertyBag, DedicatedWorkerGlobalScope, MessageEvent, Url, WorkerOptions,
};

pub(crate) struct WorkerResponder;

impl<W> Responder<W> for WorkerResponder
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    fn respond(&self, id: HandlerId, output: W::Output) {
        let msg = FromWorker::ProcessOutput(id, output);
        let data = msg.pack();
        worker_self().post_message_vec(data);
    }
}

/// Message packager, based on serde::Serialize/Deserialize
pub trait Packed {
    /// Pack serializable message into Vec<u8>
    fn pack(&self) -> Vec<u8>;
    /// Unpack deserializable message of byte slice
    fn unpack(data: &[u8]) -> Self;
}

impl<T: Serialize + for<'de> Deserialize<'de>> Packed for T {
    fn pack(&self) -> Vec<u8> {
        bincode::serialize(&self).expect("can't serialize an worker message")
    }

    fn unpack(data: &[u8]) -> Self {
        bincode::deserialize(data).expect("can't deserialize an worker message")
    }
}

/// Serializable messages to worker
#[derive(Serialize, Deserialize, Debug)]
enum ToWorker<T> {
    /// Client is connected
    Connected(HandlerId),
    /// Incoming message to Worker
    ProcessInput(HandlerId, T),
    /// Client is disconnected
    Disconnected(HandlerId),
    /// Worker should be terminated
    Destroy,
}

/// Serializable messages sent by worker to consumer
#[derive(Serialize, Deserialize, Debug)]
enum FromWorker<T> {
    /// Worker sends this message when `wasm` bundle has loaded.
    WorkerLoaded,
    /// Outgoing message to consumer
    ProcessOutput(HandlerId, T),
}

fn send_to_remote<W>(worker: &web_sys::Worker, msg: ToWorker<W::Input>)
where
    W: Worker,
    <W as Worker>::Input: Serialize + for<'de> Deserialize<'de>,
    <W as Worker>::Output: Serialize + for<'de> Deserialize<'de>,
{
    let msg = msg.pack();
    worker.post_message_vec(msg);
}
fn worker_new(
    name_of_resource: &str,
    resource_is_relative: bool,
    is_module: bool,
) -> web_sys::Worker {
    let origin = gloo_utils::document()
        .location()
        .unwrap_throw()
        .origin()
        .unwrap_throw();
    let pathname = gloo_utils::window().location().pathname().unwrap_throw();

    let prefix = if resource_is_relative {
        pathname
            .rfind(|c| c == '/')
            .map(|i| &pathname[..i])
            .unwrap_or_default()
    } else {
        ""
    };
    let script_url = format!("{}{}/{}", origin, prefix, name_of_resource);
    let wasm_url = format!(
        "{}{}/{}",
        origin,
        prefix,
        name_of_resource.replace(".js", "_bg.wasm")
    );
    let array = Array::new();
    array.push(
        &format!(
            r#"importScripts("{}");wasm_bindgen("{}");"#,
            script_url, wasm_url
        )
        .into(),
    );
    let blob = Blob::new_with_str_sequence_and_options(
        &array,
        BlobPropertyBag::new().type_("application/javascript"),
    )
    .unwrap();
    let url = Url::create_object_url_with_blob(&blob).unwrap();

    if is_module {
        let options = WorkerOptions::new();
        Reflect::set(
            options.as_ref(),
            &JsValue::from_str("type"),
            &JsValue::from_str("module"),
        )
        .unwrap();
        web_sys::Worker::new_with_options(&url, &options).expect("failed to spawn worker")
    } else {
        web_sys::Worker::new(&url).expect("failed to spawn worker")
    }
}

fn worker_self() -> DedicatedWorkerGlobalScope {
    JsValue::from(js_sys::global()).into()
}

trait WorkerExt {
    fn set_onmessage_closure(&self, handler: impl 'static + Fn(Vec<u8>));

    fn post_message_vec(&self, data: Vec<u8>);
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
        }
    )+};
}

worker_ext_impl! {
    web_sys::Worker, DedicatedWorkerGlobalScope
}