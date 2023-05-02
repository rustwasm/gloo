use std::iter::FromIterator;

use gloo_net::http::QueryParams;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn query_params_iter() {
    let mut params = QueryParams::new();
    params.append("a", "1");
    params.append("b", "value");
    let mut entries = params.iter();
    assert_eq!(entries.next(), Some(("a".into(), "1".into())));
    assert_eq!(entries.next(), Some(("b".into(), "value".into())));
    assert_eq!(entries.next(), None);
}

#[wasm_bindgen_test]
fn query_params_get() {
    let mut params = QueryParams::new();
    params.append("a", "1");
    params.append("a", "value");
    assert_eq!(params.get("a"), Some("1".to_string()));
    assert!(params.get("b").is_none());
    assert_eq!(
        params.get_all("a"),
        vec!["1".to_string(), "value".to_string()]
    );
}

#[wasm_bindgen_test]
fn query_params_delete() {
    let mut params = QueryParams::new();
    params.append("a", "1");
    params.append("a", "value");
    params.delete("a");
    assert!(params.get("a").is_none());
}

#[wasm_bindgen_test]
fn query_params_escape() {
    let mut params = QueryParams::new();
    params.append("a", "1");
    assert_eq!(params.to_string(), "a=1".to_string());

    params.append("key", "ab&c");
    assert_eq!(params.to_string(), "a=1&key=ab%26c");
}

#[wasm_bindgen_test]
fn query_clone() {
    // Verify that a deep copy is made
    let mut params1 = QueryParams::new();
    params1.append("a", "1");

    let params2 = params1.clone();
    assert_eq!(params1.get("a"), Some("1".to_string()));
    assert_eq!(params2.get("a"), Some("1".to_string()));

    params1.append("b", "2");
    assert_eq!(params1.get("b"), Some("2".to_string()));
    assert_eq!(params2.get("b"), None);
}

#[wasm_bindgen_test]
fn query_from_iter() {
    // Test from_iter
    let mut params = QueryParams::from_iter(vec![("a", "1"), ("b", "2")]);
    assert_eq!(params.get("a"), Some("1".to_string()));
    assert_eq!(params.get("b"), Some("2".to_string()));

    // Test extend
    params.extend(vec![("c", "3")]);
    assert_eq!(params.get("c"), Some("3".to_string()));
}
