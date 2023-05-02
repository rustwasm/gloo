use std::iter::FromIterator;

use gloo_net::http::Headers;
use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn headers_append() {
    let mut headers = Headers::new();
    headers.append("Content-Type", "text/plain");
    assert_eq!(headers.get("Content-Type"), Some("text/plain".to_string()));

    headers.append("Content-Type", "text/html");
    assert_eq!(headers.get("Content-Type"), Some("text/plain, text/html".to_string()));
}

#[wasm_bindgen_test]
fn headers_from_iter() {
    // Test from_iter
    let mut headers = Headers::from_iter(vec![
        ("Content-Type", "text/plain"),
        ("Content-Type", "text/html"),
        ("X-Test", "test"),
    ]);
    assert_eq!(headers.get("Content-Type"), Some("text/plain, text/html".to_string()));
    assert_eq!(headers.get("X-Test"), Some("test".to_string()));

    // Test extend
    headers.extend(vec![("X-Test", "test2")]);
    assert_eq!(headers.get("X-Test"), Some("test, test2".to_string()));
}

#[wasm_bindgen_test]
fn headers_clone() {
    // Verify that a deep copy is made
    let mut headers1 = Headers::new();
    headers1.set("Content-Type", "text/plain");

    let mut headers2 = headers1.clone();
    assert_eq!(headers1.get("Content-Type"), Some("text/plain".to_string()));
    assert_eq!(headers2.get("Content-Type"), Some("text/plain".to_string()));

    headers1.set("Content-Type", "text/html");
    assert_eq!(headers1.get("Content-Type"), Some("text/html".to_string()));
    assert_eq!(headers2.get("Content-Type"), Some("text/plain".to_string()));

    headers2.set("Content-Type", "text/css");
    assert_eq!(headers1.get("Content-Type"), Some("text/html".to_string()));
    assert_eq!(headers2.get("Content-Type"), Some("text/css".to_string()));
}
