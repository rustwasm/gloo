//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use futures::prelude::*;
use gloo_events::EventListener;
use gloo_xhr::callback::XmlHttpRequest;
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

    let load_listener = EventListener::new(request.as_ref(), "load", move |_event| {
        sender.take().map(|sender| sender.send(()).unwrap());
    });

    load_listener.forget();

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

    let error_listener = EventListener::new(request.as_ref(), "error", move |_event| {
        sender.take().map(|sender| sender.send(()).unwrap());
    });

    error_listener.forget();

    // this will trigger a CORS error.
    request.open(&http::Method::GET, "https://example.com/");

    request.send_no_body();

    receiver.map_err(|_| JsValue::from_str("onload channel was canceled"))
}
