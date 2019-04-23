//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use futures::prelude::*;
use gloo_events::EventListener;
use gloo_xhr::callback::{ReadyState, XmlHttpRequest};
use wasm_bindgen::prelude::*;
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

    request.send_without_body();

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

    request.send_without_body();

    receiver.map_err(|_| JsValue::from_str("onload channel was canceled"))
}

#[wasm_bindgen(module = "/tests/mock.js")]
extern "C" {
    #[wasm_bindgen(js_name = setXhrPrototypeResponse)]
    fn set_xhr_prototype_response(value: JsValue);

    #[wasm_bindgen(js_name = resetXhrPrototypeResponse)]
    fn reset_xhr_prototype_response();

    #[wasm_bindgen(js_name = setXhrPrototypeGetAllResponseHeaders)]
    fn set_xhr_prototype_get_all_response_headers(value: JsValue);
}

#[wasm_bindgen_test]
fn empty_response_body() {
    reset_xhr_prototype_response();
    let request = XmlHttpRequest::new();
    assert!(request.response_as_string().is_some());
    // This is the normal XHR behaviour.
    assert_eq!(String::from(request.response_as_string().unwrap()), "");
}

#[wasm_bindgen_test]
fn string_response_body() {
    set_xhr_prototype_response(JsValue::from_str("body text for tests"));
    let request = XmlHttpRequest::new();
    assert!(request.response_as_string().is_some());
    assert_eq!(request.response_as_string().unwrap(), "body text for tests");
}

#[wasm_bindgen_test]
fn bytes_response_body() {
    let array_buffer = js_sys::Uint8Array::new_with_length(6);
    array_buffer.fill(2, 0, 6);
    set_xhr_prototype_response(array_buffer.buffer().into());
    let request = XmlHttpRequest::new();
    assert!(request.response_as_bytes().is_some());
    assert_eq!(request.response_as_bytes().unwrap(), &[2, 2, 2, 2, 2, 2]);
}

#[wasm_bindgen_test]
fn default_ready_state() {
    let request = XmlHttpRequest::new();
    assert_eq!(request.ready_state(), ReadyState::Unsent);
}

#[wasm_bindgen_test]
fn get_all_response_headers() {
    set_xhr_prototype_get_all_response_headers(JsValue::from_str(
        std::str::from_utf8(b"Test: value\r\nX-My-header: something").unwrap(),
    ));
    let request = XmlHttpRequest::new();
    let mut expected = std::collections::HashMap::new();
    expected.insert("Test".to_string(), "value".to_string());
    expected.insert("X-My-header".to_string(), "something".to_string());
    assert_eq!(request.get_all_response_headers(), expected);
}
