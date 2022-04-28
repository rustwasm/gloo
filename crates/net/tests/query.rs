use gloo_net::http::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn query_params_iter() {
    let params = QueryParams::new();
    params.append("a", "1");
    params.append("b", "value");
    let mut entries = params.iter();
    assert_eq!(entries.next(), Some(("a".into(), "1".into())));
    assert_eq!(entries.next(), Some(("b".into(), "value".into())));
    assert_eq!(entries.next(), None);
}

#[wasm_bindgen_test]
fn query_params_get() {
    let params = QueryParams::new();
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
    let params = QueryParams::new();
    params.append("a", "1");
    params.append("a", "value");
    params.delete("a");
    assert!(params.get("a").is_none());
}

#[wasm_bindgen_test]
fn query_params_escape() {
    let params = QueryParams::new();
    params.append("a", "1");
    assert_eq!(params.to_string(), "a=1".to_string());

    params.append("key", "ab&c");
    assert_eq!(params.to_string(), "a=1&key=ab%26c");
}
