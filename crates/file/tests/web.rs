//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]
use futures::Future;
use wasm_bindgen_test::*;

use gloo_file::{BlobBuilder, FileReader};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
fn reading_a_blob() -> impl Future<Item = (), Error = wasm_bindgen::JsValue> {
    let reader = FileReader::new();
    let blob = BlobBuilder::new().contents("hello").build();

    reader
        .read_as_string(&blob)
        .map(|str| assert_eq!(str, "hello"))
        .map_err(|_| panic!("Errored"))
}
