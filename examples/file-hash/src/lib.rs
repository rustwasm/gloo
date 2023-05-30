use codec::TransferrableCodec;
use futures::TryStreamExt;
use gloo_worker::{HandlerId, Worker, WorkerScope};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use wasm_streams::ReadableStream;

pub mod codec;

#[derive(Serialize, Deserialize)]
pub struct HashInput {
    #[serde(skip)]
    pub file: Option<web_sys::File>,
}

#[derive(Serialize, Deserialize)]
pub struct HashOutput {
    pub hash: String,
}

pub struct HashWorker {}

impl Worker for HashWorker {
    type Input = HashInput;
    type Output = HashOutput;
    type Message = ();

    fn create(_scope: &WorkerScope<Self>) -> Self {
        Self {}
    }

    fn connected(&mut self, _scope: &WorkerScope<Self>, _id: HandlerId) {}

    fn update(&mut self, _scope: &WorkerScope<Self>, _msg: Self::Message) {}

    fn received(&mut self, scope: &WorkerScope<Self>, mut msg: Self::Input, id: HandlerId) {
        TransferrableCodec::post_decode_input(&mut msg);

        if let Some(m) = msg.file {
            let scope = scope.clone();

            spawn_local(async move {
                // This is more of a demonstration of processing transferrable types
                // than how to calculate hashes,
                // if you are trying to calculate hashes in browsers for your application,
                // please consider subtle crypto.
                // This example does not use subtle crypto
                // because calculating hashes with subtle crypto doesn't need to be sent to a worker.
                let mut hasher = Sha256::new();

                // We assume that this file is big and cannot be loaded into the memory in one chunk.
                // So we process this in chunks.
                let mut s = ReadableStream::from_raw(m.stream().unchecked_into()).into_stream();

                while let Some(chunk) = s.try_next().await.unwrap() {
                    hasher.update(chunk.unchecked_into::<Uint8Array>().to_vec());
                }

                let hash = hasher.finalize();

                let hash_hex = hex::encode(hash);

                scope.respond(id, HashOutput { hash: hash_hex });
            });
        }
    }
}
