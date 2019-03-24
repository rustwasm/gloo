//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use futures::prelude::*;
use gloo_xhr::raw::XmlHttpRequest;
use wasm_bindgen::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn constructor_does_not_throw() {
    XmlHttpRequest::new();
}

#[wasm_bindgen_test(async)]
fn minimal_request_empty_body() -> impl Future<Item = (), Error = wasm_bindgen::JsValue> {
    use futures::sync::oneshot;

    let (sender, receiver) = oneshot::channel::<()>();
    let mut sender = Some(sender);

    let request = XmlHttpRequest::new();

    request.set_onload(move |_event| {
        sender.take().map(|sender| sender.send(()).unwrap());
    });

    request.open(&http::Method::GET, "/");
    request.send_no_body();

    receiver.map_err(|_| JsValue::from_str("onload channel was canceled"))
}

#[wasm_bindgen_test(async)]
fn on_error_callback() -> impl Future<Item = (), Error = wasm_bindgen::JsValue> {
    use futures::sync::oneshot;

    let (sender, receiver) = oneshot::channel::<()>();
    let mut sender = Some(sender);

    let request = XmlHttpRequest::new();

    request.set_onerror(move |_event| {
        sender.take().map(|sender| sender.send(()).unwrap());
    });

    // this will trigger a CORS error.
    request.open(&http::Method::GET, "https://example.com/");

    request.send_no_body();

    receiver.map_err(|_| JsValue::from_str("onload channel was canceled"))
}
